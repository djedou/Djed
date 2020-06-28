pub mod console;
pub mod dialog;
pub mod fetch;
pub mod interval;
pub mod keyboard;
pub mod reader;
pub mod render;
pub mod resize;
pub mod storage;
pub mod timeout;
pub mod websocket;

#[doc(inline)]
pub use console::ConsoleService;
#[doc(inline)]
pub use dialog::DialogService;
pub use fetch::FetchService;
#[doc(inline)]
pub use interval::IntervalService;
#[doc(inline)]
pub use reader::ReaderService;
#[doc(inline)]
pub use render::RenderService;
#[doc(inline)]
pub use resize::ResizeService;
#[doc(inline)]
pub use storage::StorageService;
#[doc(inline)]
pub use timeout::TimeoutService;
#[doc(inline)]
pub use websocket::WebSocketService;

use std::time::Duration;

/// An universal task of a service.
/// The task must be handled when it is cancelled.
pub trait Task: Drop {
    /// Returns `true` if task is active.
    fn is_active(&self) -> bool;
}

#[doc(hidden)]
/// Converts a `Duration` into milliseconds.
fn to_ms(duration: Duration) -> u32 {
    let ms = duration.subsec_millis();
    ms + duration.as_secs() as u32 * 1000
}