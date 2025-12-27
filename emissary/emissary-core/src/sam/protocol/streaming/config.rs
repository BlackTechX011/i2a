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

#![allow(unused)]

use alloc::string::String;
use core::{num::NonZeroUsize, time::Duration};

/// Inactivity action.
#[derive(Debug)]
pub enum InactivityAction {
    /// Do nothing,
    DoNothing,

    /// Disconnect remote router.
    Disconnect,

    /// Send duplicate ACK.
    Send,
}

/// Limit action.
#[derive(Debug)]
pub enum LimitAction {
    /// Reset connection.
    Reset,

    /// Drop connection.
    Drop,

    /// Send HTTP 429 status code.
    Http,
}

/// Profile for the streaming application.
///
/// See section `i2p.streaming.profile Notes` in the docs [1]
///
/// [1]: https://geti2p.net/en/docs/api/streaming
#[derive(Debug)]
pub enum Profile {
    /// Bulk.
    Bulk,

    /// Interactive.
    Interactive,
}

/// Streaming protocol configuration.
#[derive(Debug)]
pub struct StreamConfig {
    /// Whether to respond to incoming pings
    pub answer_pings: bool,

    /// Comma- or space-separated list of Base64 peer Hashes to be blacklisted for incoming
    /// connections to ALL destinations in the context. This option must be set in the context
    /// properties, NOT in the createManager() options argument. Note that setting this in the
    /// router context will not affect clients outside the router in a separate JVM and context. As
    /// of release 0.9.3.
    pub blacklist: String,

    /// How much transmit data (in bytes) will be accepted that hasn't been written out yet.
    pub buffer_size: usize,

    /// When we're in congestion avoidance, we grow the window size at the rate of
    /// 1/(windowSize*factor). In standard TCP, window sizes are in bytes, while in I2P, window
    /// sizes are in messages. A higher number means slower growth.
    pub congestion_avoidance_growth_rate_factor: usize,

    /// How long to wait after instantiating a new con before actually attempting to connect. If
    /// this is <= 0, connect immediately with no initial data. If greater than 0, wait until the
    /// output stream is flushed, the buffer fills, or that many milliseconds pass, and include any
    /// initial data with the SYN.
    pub connect_delay: Option<Duration>,

    /// How long to block on connect, in milliseconds. Negative means indefinitely. Default is 5
    /// minutes.
    pub connect_timeout: Option<Duration>,

    /// Comma- or space-separated list of Base64 peer Hashes or host names to be contacted using an
    /// alternate DSA destination. Only applies if multisession is enabled and the primary session
    /// is non-DSA (generally for shared clients only). This option must be set in the context
    /// properties, NOT in the createManager() options argument. Note that setting this in the
    /// router context will not affect clients outside the router in a separate JVM and context. As
    /// of release 0.9.21.
    pub dsa_list: String,

    /// Whether to listen only for the streaming protocol. Setting to true will prohibit
    /// communication with Destinations earlier than release 0.7.1 (released March 2009). Set to
    /// true if running multiple protocols on this Destination. As of release 0.9.1. Default true
    /// as of release 0.9.36.
    pub enforce_protocol: bool,

    /// (send)  (0=noop, 1=disconnect) What to do on an inactivity timeout - do nothing,
    /// disconnect, or send a duplicate ack.
    pub inactivity_action: InactivityAction,

    /// Idle time before sending a keepalive
    pub inactivity_timeout: Duration,

    /// Delay before sending an ack
    pub initial_ack_delay: Duration,

    /// The initial value of the resend delay field in the packet header, times 1000. Not fully
    /// implemented; see below.
    pub initial_resend_delay: Duration,

    /// Initial timeout (if no sharing data available). As of release 0.9.8.
    pub initial_rto: Duration,

    /// Initial round trip time estimate (if no sharing data available). Disabled as of release
    /// 0.9.8; uses actual RTT.
    pub initial_rtt: Duration,

    /// (if no sharing data available) In standard TCP, window sizes are in bytes, while in I2P,
    /// window sizes are in messages.
    pub initial_window_size: usize,

    /// What action to take when an incoming connection exceeds limits. Valid values are: reset
    /// (reset the connection); drop (drop the connection); or http (send a hardcoded HTTP 429
    /// response). Any other value is a custom response to be sent. backslash-r and backslash-n
    /// will be replaced with CR and LF. As of release 0.9.34.
    pub limit_action: LimitAction,

    /// (0 or negative value means unlimited) This is a total limit for incoming and outgoing
    /// combined.
    pub max_concurrent_streams: Option<NonZeroUsize>,

    /// Incoming connection limit (per peer; 0 means disabled) As of release 0.7.14.
    pub max_conns_per_minute: Option<NonZeroUsize>,

    /// (per peer; 0 means disabled) As of release 0.7.14.
    pub max_conns_per_hour: Option<NonZeroUsize>,

    /// (per peer; 0 means disabled) As of release 0.7.14.
    pub max_conns_per_day: Option<NonZeroUsize>,

    /// The max_Imum size of the payload, i.e. the MTU in bytes.
    pub max_message_size: usize,

    /// Max_Imum number of retransmissions before failure.
    pub max_resends: usize,

    /// Incoming connection limit (all peers; 0 means disabled) As of release 0.7.14.
    pub max_total_conns_per_minute: Option<NonZeroUsize>,

    /// (all peers; 0 means disabled) Use with caution as exceeding this will disable a server for
    /// a long time. As of release 0.7.14.
    pub max_total_conns_per_hour: Option<NonZeroUsize>,

    /// (all peers; 0 means disabled) Use with caution as exceeding this will disable a server for
    /// a long time. As of release 0.7.14.
    pub max_total_conns_per_day: Option<NonZeroUsize>,

    /// Maximum window size.
    pub max_window_size: usize,

    /// Streaming application profile.
    pub profile: Profile,

    /// How long to block on read, in milliseconds. Negative means indefinitely.
    pub read_timeout: Option<NonZeroUsize>,

    /// When we're in slow start, we grow the window size at the rate of 1/(factor). In standard
    /// TCP, window sizes are in bytes, while in I2P, window sizes are in messages. A higher number
    /// means slower growth.
    pub slow_start_growth_rate_factor: usize,

    /// Ref: RFC 2140. Floating point value. May be set only via context properties, not connection
    /// options. As of release 0.9.8.
    pub rtt_dampening: f64,

    /// Ref: RFC 2140. Floating point value. May be set only via context properties, not connection
    /// options. As of release 0.9.8.
    pub rttdev_dampening: f64,

    /// Ref: RFC 2140. Floating point value. May be set only via context properties, not connection
    /// options. As of release 0.9.8.
    pub wdw_dampening: f64,

    /// How long to block on write/flush, in milliseconds. Negative means indefinitely.
    pub write_timeout: Option<NonZeroUsize>,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            answer_pings: true,
            blacklist: String::from(""),
            buffer_size: 64 * 1000,
            congestion_avoidance_growth_rate_factor: 1,
            connect_delay: None,
            connect_timeout: Some(Duration::from_secs(5 * 60)),
            dsa_list: String::from(""),
            enforce_protocol: true,
            inactivity_action: InactivityAction::Send,
            inactivity_timeout: Duration::from_secs(90),
            initial_ack_delay: Duration::from_millis(750),
            initial_resend_delay: Duration::from_secs(1),
            initial_rto: Duration::from_secs(9),
            initial_rtt: Duration::from_secs(8),
            initial_window_size: 6,
            limit_action: LimitAction::Reset,
            max_concurrent_streams: None,
            max_conns_per_minute: None,
            max_conns_per_hour: None,
            max_conns_per_day: None,
            max_message_size: 1730,
            max_resends: 8,
            max_total_conns_per_minute: None,
            max_total_conns_per_hour: None,
            max_total_conns_per_day: None,
            max_window_size: 12,
            profile: Profile::Bulk,
            read_timeout: None,
            slow_start_growth_rate_factor: 1,
            rtt_dampening: 0.75f64,
            rttdev_dampening: 0.75f64,
            wdw_dampening: 0.75f64,
            write_timeout: None,
        }
    }
}
