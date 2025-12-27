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

use crate::primitives::TunnelId;

use bytes::{BufMut, BytesMut};
use nom::{
    bytes::complete::take,
    number::complete::{be_u16, be_u32},
    IResult,
};

use alloc::vec::Vec;

/// Tunnel gateway message.
pub struct TunnelGateway<'a> {
    /// Tunnel ID.
    pub tunnel_id: TunnelId,

    /// Payload.
    pub payload: &'a [u8],
}

impl<'a> TunnelGateway<'a> {
    /// Attempt to parse `TunnelGateway` from `input`.
    ///
    /// Returns the parsed message and rest of `input` on success.
    fn parse_frame(input: &'a [u8]) -> IResult<&'a [u8], TunnelGateway<'a>> {
        let (rest, tunnel_id) = be_u32(input)?;
        let (rest, size) = be_u16(rest)?;
        let (rest, payload) = take(size as usize)(rest)?;

        Ok((
            rest,
            TunnelGateway {
                tunnel_id: TunnelId::from(tunnel_id),
                payload,
            },
        ))
    }

    /// Attempt to parse `input` into `TunnelGateway`.
    pub fn parse(input: &'a [u8]) -> Option<TunnelGateway<'a>> {
        Some(Self::parse_frame(input).ok()?.1)
    }

    /// Serialize `TunnelGateway` into a byte vector.
    pub fn serialize(self) -> Vec<u8> {
        let mut out = BytesMut::with_capacity(self.payload.len() + 2 + 4);

        out.put_u32(*self.tunnel_id);
        out.put_u16(self.payload.len() as u16);
        out.put_slice(self.payload);

        out.freeze().to_vec()
    }
}
