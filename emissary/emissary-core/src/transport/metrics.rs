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

use crate::runtime::MetricType;

use alloc::vec::Vec;

pub const NUM_CONNECTIONS: &str = "connections_count";
pub const NUM_INBOUND: &str = "inbound_connections_count";
pub const NUM_OUTBOUND: &str = "outbound_connections_count";
pub const NUM_REJECTED: &str = "rejected_connections_count";
pub const NUM_DIAL_FAILURES: &str = "dial_failure_count";

/// Register transport metrics.
pub fn register_metrics(mut metrics: Vec<MetricType>) -> Vec<MetricType> {
    // counters
    metrics.push(MetricType::Counter {
        name: NUM_INBOUND,
        description: "total number of inbound connections",
    });
    metrics.push(MetricType::Counter {
        name: NUM_OUTBOUND,
        description: "total number of outbound connections",
    });
    metrics.push(MetricType::Counter {
        name: NUM_DIAL_FAILURES,
        description: "total number of dial failures",
    });
    metrics.push(MetricType::Counter {
        name: NUM_REJECTED,
        description: "total number of rejected connections",
    });

    // gauges
    metrics.push(MetricType::Gauge {
        name: NUM_CONNECTIONS,
        description: "number of active connections",
    });

    metrics
}
