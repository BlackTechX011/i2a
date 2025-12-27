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

use crate::{
    i2cp::message::{MessageType, I2CP_HEADER_SIZE},
    primitives::Lease,
};

use bytes::{BufMut, BytesMut};

use alloc::vec::Vec;

/// `RequestVariableLeaseSet` message.
///
/// https://geti2p.net/spec/i2cp#requestvariableleasesetmessage
pub struct RequestVariableLeaseSet(());

impl RequestVariableLeaseSet {
    /// Create new `RequestVariableLeaseSet` message.
    ///
    /// Caller must have made user `leases` contains at least one `Lease`.
    pub fn new(session_id: u16, leases: Vec<Lease>) -> BytesMut {
        let payload_len = 1 + leases.len() * leases[0].serialized_len_lease();
        let mut out = BytesMut::with_capacity(I2CP_HEADER_SIZE + payload_len);

        out.put_u32(payload_len as u32);
        out.put_u8(MessageType::RequestVariableLeaseSet.as_u8());
        out.put_u16(session_id);
        out.put_u8(leases.len() as u8);

        for lease in leases {
            out.put_slice(&lease.serialize_lease());
        }

        out
    }
}
