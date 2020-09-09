//! Service to send HTTP-request to a server.

mod fetch;

pub use fetch::{Cache, Credentials, Mode, Redirect, Window, WorkerGlobalScope,
    HeaderMap, Method, Request, Response, StatusCode, Uri, FetchOptions, FetchTask,
    FetchService
};
//pub use self::web_sys::*;

/// Type to set referrer for fetch.
#[derive(Debug)]
pub enum Referrer {
    /// `<same-origin URL>` value of referrer.
    SameOriginUrl(String),
    /// `about:client` value of referrer.
    AboutClient,
    /// `<empty string>` value of referrer.
    Empty,
}
