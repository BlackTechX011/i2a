// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use crate::{protocol::Protocol, runtime::Runtime};

use alloc::vec::Vec;
use core::marker::PhantomData;

/// I2CP payload builder.
///
/// Compresses the payload and modifies the gzip header to include I2CP protocol fields.
pub struct I2cpPayloadBuilder<'a, R: Runtime> {
    /// Destination port.
    dst_port: u16,

    /// Payload.
    payload: &'a [u8],

    /// Protocol.
    protocol: Option<Protocol>,

    /// Source port.
    src_port: u16,

    /// Marker for `Runtime`.
    _runtime: PhantomData<R>,
}

impl<'a, R: Runtime> I2cpPayloadBuilder<'a, R> {
    /// Create new [`GzipEncoderBuilder`].
    pub fn new(payload: &'a [u8]) -> Self {
        Self {
            dst_port: 0u16,
            payload,
            protocol: None,
            src_port: 0u16,
            _runtime: Default::default(),
        }
    }

    /// Specify source port.
    ///
    /// Optional, defaults to 0.
    #[allow(unused)]
    pub fn with_source_port(mut self, src_port: u16) -> Self {
        self.src_port = src_port;
        self
    }

    /// Specify destination port.
    ///
    /// Optional, defaults to 0.
    #[allow(unused)]
    pub fn with_destination_port(mut self, dst_port: u16) -> Self {
        self.dst_port = dst_port;
        self
    }

    /// Specify protocol.
    pub fn with_protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = Some(protocol);
        self
    }

    /// Compress payload and modify the gzip payload.
    pub fn build(mut self) -> Option<Vec<u8>> {
        R::gzip_compress(self.payload).map(|mut compressed| {
            compressed[4..6].copy_from_slice(&self.src_port.to_be_bytes());
            compressed[6..8].copy_from_slice(&self.dst_port.to_be_bytes());
            compressed[9] = self.protocol.take().expect("protocol to exist").as_u8();

            compressed
        })
    }
}

/// I2CP payload.
///
/// Decompresses the payload and extracts I2CP-related fields from header.
pub struct I2cpPayload {
    /// Destination port.
    pub dst_port: u16,

    /// Decompressed payload.
    pub payload: Vec<u8>,

    /// Protocol.
    pub protocol: Protocol,

    /// Source port.
    pub src_port: u16,
}

impl I2cpPayload {
    /// Extract I2CP-related fields from gzip header and decompress `payload`.
    pub fn decompress<R: Runtime>(payload: impl AsRef<[u8]>) -> Option<Self> {
        if payload.as_ref().len() < 10 {
            return None;
        }

        // `TryInto::try_into()` must succeed as `payload` is guaranteed to have enough bytes
        // and slices taken of the payload are of correct size for a `u16`
        let src_port = TryInto::<[u8; 2]>::try_into(&payload.as_ref()[4..6]).expect("to succeed");
        let dst_port = TryInto::<[u8; 2]>::try_into(&payload.as_ref()[6..8]).expect("to succeed");
        let protocol = payload.as_ref()[9];

        Some(Self {
            src_port: u16::from_be_bytes(src_port),
            dst_port: u16::from_be_bytes(dst_port),
            protocol: Protocol::from_u8(protocol)?,
            payload: R::gzip_decompress(&payload)?,
        })
    }
}

/// I2CP protocol parameters.
///
/// Extracted from a GZIP-encompressed payload.
#[derive(Debug, PartialEq, Eq)]
pub struct I2cpParameters {
    /// Destination port.
    pub dst_port: u16,

    /// Protocol.
    pub protocol: Protocol,

    /// Source port.
    pub src_port: u16,
}

impl I2cpParameters {
    /// Create new [`I2cpParameters`] from `payload`.
    pub fn new(payload: impl AsRef<[u8]>) -> Option<I2cpParameters> {
        if payload.as_ref().len() < 10 {
            return None;
        }

        // `TryInto::try_into()` must succeed as `payload` is guaranteed to have enough bytes
        // and slices taken of the payload are of correct size for a `u16`
        let src_port = TryInto::<[u8; 2]>::try_into(&payload.as_ref()[4..6]).expect("to succeed");
        let dst_port = TryInto::<[u8; 2]>::try_into(&payload.as_ref()[6..8]).expect("to succeed");
        let protocol = Protocol::from_u8(payload.as_ref()[9])?;

        Some(Self {
            dst_port: u16::from_be_bytes(dst_port),
            protocol,
            src_port: u16::from_be_bytes(src_port),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{mock::MockRuntime, Runtime};

    #[test]
    fn compress_and_decompress() {
        let payload = "hello, world".as_bytes();

        let compressed = I2cpPayloadBuilder::<MockRuntime>::new(&payload)
            .with_source_port(13)
            .with_protocol(Protocol::Streaming)
            .build()
            .unwrap();

        // normal compression works
        let decompressed = MockRuntime::gzip_decompress(&compressed).unwrap();
        assert_eq!(decompressed, "hello, world".as_bytes());

        // extracing i2cp data + decompression works
        let Some(I2cpPayload {
            dst_port,
            payload,
            protocol,
            src_port,
        }) = I2cpPayload::decompress::<MockRuntime>(&compressed)
        else {
            panic!("invalid data");
        };

        assert_eq!(dst_port, 0u16);
        assert_eq!(src_port, 13u16);
        assert_eq!(protocol, Protocol::Streaming);
        assert_eq!(payload, "hello, world".as_bytes());
    }

    #[test]
    fn invalid_protocol() {
        let payload = "hello, world".as_bytes();

        let mut compressed = I2cpPayloadBuilder::<MockRuntime>::new(&payload)
            .with_source_port(13)
            .with_protocol(Protocol::Streaming)
            .build()
            .unwrap();

        compressed[9] = 0xaa;

        assert!(I2cpPayload::decompress::<MockRuntime>(&compressed).is_none());
        assert!(I2cpParameters::new(&compressed).is_none());
    }

    #[test]
    fn extract_parameters() {
        let payload = "hello, world".as_bytes();

        let compressed = I2cpPayloadBuilder::<MockRuntime>::new(&payload)
            .with_source_port(13)
            .with_destination_port(37)
            .with_protocol(Protocol::Anonymous)
            .build()
            .unwrap();

        assert_eq!(
            I2cpParameters::new(&compressed),
            Some(I2cpParameters {
                dst_port: 37,
                src_port: 13,
                protocol: Protocol::Anonymous
            })
        );
    }
}
