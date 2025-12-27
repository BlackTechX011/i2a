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

//! Tunnel expiration timer.
//!
//! [`TunnelTimer`] emits two signals for each tunnel of the pool:
//!  a) signal when `TunnelPool` should start building a new tunnel
//!  b) signal when the tunnel expires
//!
//! The first signal is emitted 2 minutes before the tunnel expires and provides `TunnelPool` with
//! some time budget to build a new tunnel to replace the tunnel that's about to expire. The second
//! signal is emitted after the tunnel has been active for 10 minutes, meaning it cannot be used for
//! any tunnel message transportation anymore.
//!
//! Inbound and outbound tunnels work differently in the sense that inbound tunnels have a dedicated
//! asynchronous event loop which they're responsible for polling themselves. This event loop also
//! contains a timer, allowing the inbound tunnel to shut itself down after 10 minutes have passed.
//! This means that `TunnelTimer` doesn't have to emit [`TunnelTimerEvent::Destroy`] for inbound
//! tunnels as the destruction happens automatically and the event is only emitted for outbound
//! tunnels which do not have an asynchronous event loop.
//!
//! [`TunnelTimerEvent::Rebuild`] is emitted for both tunnel types.

use crate::{
    primitives::TunnelId,
    runtime::{JoinSet, Runtime},
    tunnel::TUNNEL_EXPIRATION,
};

use futures::{Stream, StreamExt};

use core::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

/// Tunnel rebuild timeout.
///
/// Tunnel of a pool needs to be rebuilt before it expires as otherwise the pool may be not have any
/// tunnels of that type. Start building a new tunnel to replace to old one 2 minutes before the old
/// tunnel expires.
const TUNNEL_REBUILD_TIMEOUT: Duration = Duration::from_secs(9 * 60);

/// Tunnel kind.
#[derive(Clone, Copy)]
pub enum TunnelKind {
    /// Outbound tunnel.
    Outbound {
        /// Tunnel ID.
        tunnel_id: TunnelId,
    },

    /// Inbound tunnel.
    Inbound {
        /// Tunnel ID.
        tunnel_id: TunnelId,
    },
}

/// Events emitted by [`TunnelTimer`].
pub enum TunnelTimerEvent {
    /// Rebuild a tunnel.
    Rebuild {
        /// Tunnel kind and it's `TunnelId`.
        kind: TunnelKind,
    },

    /// Destroy an expired outbound tunnel.
    Destroy {
        /// ID of the expired outbound tunnel.
        tunnel_id: TunnelId,
    },
}

/// Tunnel timer
pub struct TunnelTimer<R: Runtime> {
    /// Pending tunnel timers.
    timers: R::JoinSet<TunnelTimerEvent>,
}

impl<R: Runtime> TunnelTimer<R> {
    /// Create new [`TunnelTimer`].
    pub fn new() -> Self {
        Self {
            timers: R::join_set(),
        }
    }

    /// Add timers for an inbound tunnel.
    pub fn add_inbound_tunnel(&mut self, tunnel_id: TunnelId) {
        self.timers.push(async move {
            R::delay(TUNNEL_REBUILD_TIMEOUT).await;
            TunnelTimerEvent::Rebuild {
                kind: TunnelKind::Inbound { tunnel_id },
            }
        });
    }

    /// Add timers for an outbound tunnel.
    pub fn add_outbound_tunnel(&mut self, tunnel_id: TunnelId) {
        self.timers.push(async move {
            R::delay(TUNNEL_REBUILD_TIMEOUT).await;
            TunnelTimerEvent::Rebuild {
                kind: TunnelKind::Outbound { tunnel_id },
            }
        });
        self.timers.push(async move {
            R::delay(TUNNEL_EXPIRATION).await;
            TunnelTimerEvent::Destroy { tunnel_id }
        });
    }
}

impl<R: Runtime> Stream for TunnelTimer<R> {
    type Item = TunnelTimerEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.timers.poll_next_unpin(cx)
    }
}
