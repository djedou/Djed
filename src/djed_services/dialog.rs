//! This module contains the implementation of a service
//! to show alerts and confirm dialogs in a browser.

use crate::utils;

/// A dialog service.
#[derive(Default, Debug)]
pub struct DialogService {}

impl DialogService {
    /// Calls [alert](https://developer.mozilla.org/en-US/docs/Web/API/Window/alert)
    /// function.
    pub fn alert(message: &str) {
        utils::window().alert_with_message(message).unwrap();
    }

    /// Calls [confirm](https://developer.mozilla.org/en-US/docs/Web/API/Window/confirm)
    /// function.
    pub fn confirm(message: &str) -> bool {
        utils::window().confirm_with_message(message).unwrap()
    }
}