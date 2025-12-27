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

use crate::i2cp::message::{MessageType, SessionId, I2CP_HEADER_SIZE, LOG_TARGET};

use bytes::{BufMut, BytesMut};

/// Session status kind.
#[derive(Debug)]
pub enum SessionStatusKind {
    /// Session destroyed.
    Destroyed,

    /// Session crated.
    Created,

    /// Session updated.
    #[allow(unused)]
    Updated,

    /// Invalid session.
    #[allow(unused)]
    Invalid,

    /// Session refused.
    Refused,
}

impl SessionStatusKind {
    /// Serialize [`SessionStatusKind`].
    pub fn as_u8(self) -> u8 {
        match self {
            Self::Destroyed => 0,
            Self::Created => 1,
            Self::Updated => 2,
            Self::Invalid => 3,
            Self::Refused => 4,
        }
    }

    /// Try to convert `status_kind` into `SessionStatusKind`.
    #[allow(unused)]
    pub fn from_u8(kind: u8) -> Option<Self> {
        match kind {
            0 => Some(Self::Destroyed),
            1 => Some(Self::Created),
            2 => Some(Self::Updated),
            3 => Some(Self::Invalid),
            4 => Some(Self::Refused),
            kind => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?kind,
                    "invalid session status",
                );
                None
            }
        }
    }
}

/// `SessionStatus` message.
///
/// https://geti2p.net/spec/i2cp#sessionstatusmessage
pub struct SessionStatus(());

impl SessionStatus {
    /// Create new `SessionStatus` message.
    pub fn new(session_id: SessionId, kind: SessionStatusKind) -> BytesMut {
        let mut out = BytesMut::with_capacity(I2CP_HEADER_SIZE + 3);

        out.put_u32(3u32);
        out.put_u8(MessageType::SessionStatus.as_u8());
        out.put_u16(session_id.as_u16());
        out.put_u8(kind.as_u8());

        out
    }
}
