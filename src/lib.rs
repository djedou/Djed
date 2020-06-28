pub mod djed;
pub mod djed_dom;
pub mod callback;
pub mod scheduler;
pub mod utils;
pub mod djed_agent;
pub mod djed_format;
pub mod djed_services;


use proc_macro_hack::proc_macro_hack;
/// This macro implements JSX-like templates.
#[proc_macro_hack(support_nested)]
pub use djed_macros::html;

#[proc_macro_hack(support_nested)]
pub use djed_macros::html_nested;


/// This module contains macros which implements html! macro and JSX-like templates
/*pub mod macros {
    pub use crate::html;
    pub use crate::html_nested;
    pub use djed_macros::Properties;
}*/

/// The module that contains all events available in the framework.
pub mod events {
    pub use crate::djed::listener::{ChangeData, InputData};

    pub use web_sys::{
        AnimationEvent, DragEvent, ErrorEvent, Event, FocusEvent, InputEvent, KeyboardEvent,
        MouseEvent, PointerEvent, ProgressEvent, TouchEvent, TransitionEvent, UiEvent, WheelEvent,
    };

}

