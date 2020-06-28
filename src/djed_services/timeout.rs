use super::{to_ms, Task};
use crate::callback::Callback;
use std::fmt;
use std::time::Duration;
use gloo::timers::callback::Timeout;


/// A handle to cancel a timeout task.
#[must_use]
pub struct TimeoutTask(Option<Timeout>);

impl fmt::Debug for TimeoutTask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TimeoutTask")
    }
}

/// An service to set a timeout.
#[derive(Default, Debug)]
pub struct TimeoutService {}

impl TimeoutService {
    /// Sets timeout which sends messages from a `converter` after `duration`.
    pub fn spawn(duration: Duration, callback: Callback<()>) -> TimeoutTask {
        let callback = move || {
            callback.emit(());
        };
        let ms = to_ms(duration);
        let handle = Timeout::new(ms, callback);
        TimeoutTask(Some(handle))
    }
}

impl Task for TimeoutTask {
    fn is_active(&self) -> bool {
        self.0.is_some()
    }
}

impl Drop for TimeoutTask {
    fn drop(&mut self) {

    }
}