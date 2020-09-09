use super::Task;
use crate::callback::Callback;
use crate::djed_format::{Binary, FormatError, Text};
use std::fmt;
use gloo::events::EventListener;
use js_sys::Uint8Array;
use wasm_bindgen::JsCast;
use web_sys::{BinaryType, Event, MessageEvent, WebSocket};

/// A status of a websocket connection. Used for status notification.
#[derive(Clone, Debug, PartialEq)]
pub enum WebSocketStatus {
    /// Fired when a websocket connection was opened.
    Opened,
    /// Fired when a websocket connection was closed.
    Closed,
    /// Fired when a websocket connection was failed.
    Error,
}

/// A handle to control current websocket connection. Implements `Task` and could be canceled.
#[must_use]
pub struct WebSocketTask {
    ws: WebSocket,
    notification: Callback<WebSocketStatus>,
    #[allow(dead_code)]
    listeners: [EventListener; 4],
}

impl WebSocketTask {
    fn new(
        ws: WebSocket,
        notification: Callback<WebSocketStatus>,
        listener_0: EventListener,
        listeners: [EventListener; 3],
    ) -> Result<WebSocketTask, &'static str> {
        let [listener_1, listener_2, listener_3] = listeners;
        Ok(WebSocketTask {
            ws,
            notification,
            listeners: [listener_0, listener_1, listener_2, listener_3],
        })
    }
}

impl fmt::Debug for WebSocketTask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("WebSocketTask")
    }
}

/// A websocket service attached to a user context.
#[derive(Default, Debug)]
pub struct WebSocketService {}

impl WebSocketService {
    /// Connects to a server by a websocket connection. Needs two functions to generate
    /// data and notification messages.
    pub fn connect<OUT: 'static>(
        url: &str,
        callback: Callback<OUT>,
        notification: Callback<WebSocketStatus>,
    ) -> Result<WebSocketTask, &str>
    where
        OUT: From<Text> + From<Binary>,
    {
        let ConnectCommon(ws, listeners) = Self::connect_common(url, &notification)?;
        let listener = EventListener::new(&ws, "message", move |event: &Event| {
            let event = event.dyn_ref::<MessageEvent>().unwrap();
            process_both(&event, &callback);
        });
        WebSocketTask::new(ws, notification, listener, listeners)

    }

    /// Connects to a server by a websocket connection, like connect,
    /// but only processes binary frames. Text frames are silently
    /// ignored. Needs two functions to generate data and notification
    /// messages.
    pub fn connect_binary<OUT: 'static>(
        url: &str,
        callback: Callback<OUT>,
        notification: Callback<WebSocketStatus>,
    ) -> Result<WebSocketTask, &str>
    where
        OUT: From<Binary>,
    {
        let ConnectCommon(ws, listeners) = Self::connect_common(url, &notification)?;
        let listener = EventListener::new(&ws, "message", move |event: &Event| {
            let event = event.dyn_ref::<MessageEvent>().unwrap();
            process_binary(&event, &callback);
        });
        WebSocketTask::new(ws, notification, listener, listeners)
    }

    /// Connects to a server by a websocket connection, like connect,
    /// but only processes text frames. Binary frames are silently
    /// ignored. Needs two functions to generate data and notification
    /// messages.
    pub fn connect_text<OUT: 'static>(
        url: &str,
        callback: Callback<OUT>,
        notification: Callback<WebSocketStatus>,
    ) -> Result<WebSocketTask, &str>
    where
        OUT: From<Text>,
    {
        let ConnectCommon(ws, listeners) = Self::connect_common(url, &notification)?;
        let listener = EventListener::new(&ws, "message", move |event: &Event| {
            let event = event.dyn_ref::<MessageEvent>().unwrap();
            process_text(&event, &callback);
        });
        WebSocketTask::new(ws, notification, listener, listeners)
    }

    fn connect_common(
        url: &str,
        notification: &Callback<WebSocketStatus>,
    ) -> Result<ConnectCommon, &'static str> {
        let ws = WebSocket::new(url);
        if ws.is_err() {
            return Err("Failed to created websocket with given URL");
        }

        let ws = ws.map_err(|_| "failed to build websocket")?;
        ws.set_binary_type(BinaryType::Arraybuffer);
        let notify = notification.clone();
        let listener_open =
            move |_: &Event| {
                notify.emit(WebSocketStatus::Opened);
            };
        let notify = notification.clone();
        let listener_close =
            move |_: &Event| {
                notify.emit(WebSocketStatus::Closed);
            };
        let notify = notification.clone();
        let listener_error =
            move |_: &Event| {
                notify.emit(WebSocketStatus::Error);
            };
        {
            let listeners = [
                    EventListener::new(&ws, "open", listener_open),
                    EventListener::new(&ws, "close", listener_close),
                    EventListener::new(&ws, "error", listener_error),
                ];
            Ok(ConnectCommon(
                ws,
                listeners,
            ))
        }
    }
}

struct ConnectCommon(WebSocket, [EventListener; 3]);

fn process_binary<OUT: 'static>(event: &MessageEvent, callback: &Callback<OUT>) where
    OUT: From<Binary>,
{
    let bytes = if !event.data().is_string() {
        Some(event.data())
    } else {
        None
    };

    let data = if let Some(bytes) = bytes {
        let bytes: Vec<u8> = Uint8Array::new(&bytes).to_vec();
        Ok(bytes)
    } else {
        Err(FormatError::ReceivedTextForBinary.into())
    };

    let out = OUT::from(data);
    callback.emit(out);
}

fn process_text<OUT: 'static>(event: &MessageEvent, callback: &Callback<OUT>)
 where
    OUT: From<Text>,
{
    let text = event.data().as_string();

    let data = if let Some(text) = text {
        Ok(text)
    } else {
        Err(FormatError::ReceivedBinaryForText.into())
    };

    let out = OUT::from(data);
    callback.emit(out);
}

fn process_both<OUT: 'static>(
    event: &MessageEvent,
    callback: &Callback<OUT>,
) where
    OUT: From<Text> + From<Binary>,
{
    let is_text = event.data().is_string();

    if is_text {
        process_text(event, callback);
    } else {
        process_binary(event, callback);
    }
}

impl WebSocketTask {
    /// Sends data to a websocket connection.
    pub fn send<IN>(&mut self, data: IN)
    where
        IN: Into<Text>,
    {
        if let Ok(body) = data.into() {
            let result = self.ws.send_with_str(&body);
            if result.is_err() {
                self.notification.emit(WebSocketStatus::Error);
            }
        }
    }

    /// Sends binary data to a websocket connection.
    pub fn send_binary<IN>(&mut self, data: IN)
    where
        IN: Into<Binary>,
    {
        if let Ok(body) = data.into() {
            let result = self.ws.send_with_u8_array(&body);

            if result.is_err() {
                self.notification.emit(WebSocketStatus::Error);
            }
        }
    }
}

impl Task for WebSocketTask {
    fn is_active(&self) -> bool {
        self.ws.ready_state() == WebSocket::OPEN
    }
}

impl Drop for WebSocketTask {
    fn drop(&mut self) {
        if self.is_active() {
            self.ws.close().ok();
        }
    }
}
