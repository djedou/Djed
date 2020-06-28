use crate::djed_services::{to_ms, Task};
use crate::callback::Callback;
use std::fmt;
use std::time::Duration;
use gloo::timers::callback::Interval;


/// A handle which helps to cancel interval. Uses
/// [clearInterval](https://developer.mozilla.org/en-US/docs/Web/API/WindowOrWorkerGlobalScope/clearInterval).
#[must_use]
pub struct IntervalTask(Interval);

impl fmt::Debug for IntervalTask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("IntervalTask")
    }
}

/// A service to send messages on every elapsed interval.
#[derive(Default, Debug)]
pub struct IntervalService {}

impl IntervalService {
    /// Sets interval which will call send a messages returned by a converter
    /// on every interval expiration.
    pub fn spawn(duration: Duration, callback: Callback<()>) -> IntervalTask {
        let callback = move || {
            callback.emit(());
        };
        let ms = to_ms(duration);
        let handle = Interval::new(ms, callback);
        IntervalTask(handle)
    }
}

impl Task for IntervalTask {
    fn is_active(&self) -> bool {
        true
    }
}

impl Drop for IntervalTask {
    fn drop(&mut self) {
    }
}

