use std::io;
use serde_json;
use websocket::result::WebSocketError;
use websocket::client::ParseError;
use websocket::futures::sync::mpsc::SendError;
use websocket::OwnedMessage;


#[derive(Debug)]
pub enum MessageError {
    SendError(SendError<OwnedMessage>),
    WebSocket(WebSocketError),
    Json(serde_json::Error),
}

impl From<SendError<OwnedMessage>> for MessageError {
    fn from(e: SendError<OwnedMessage>) -> Self {
        return MessageError::SendError(e);
    }
}

impl From<WebSocketError> for MessageError {
    fn from(e: WebSocketError) -> Self {
        return MessageError::WebSocket(e);
    }
}

impl From<serde_json::Error> for MessageError {
    fn from(e: serde_json::Error) -> Self {
        return MessageError::Json(e);
    }
}


#[derive(Debug)]
pub enum ConnectError {
    WebSocket(WebSocketError),
    Parse(ParseError),
    IO(io::Error),
}

impl From<WebSocketError> for ConnectError {
    fn from(e: WebSocketError) -> Self {
        return ConnectError::WebSocket(e);
    }
}

impl From<ParseError> for ConnectError {
    fn from(e: ParseError) -> Self {
        return ConnectError::Parse(e);
    }
}

impl From<io::Error> for ConnectError {
    fn from(e: io::Error) -> Self {
        return ConnectError::IO(e);
    }
}


#[derive(Debug)]
pub enum JoinError {
    SendError(SendError<OwnedMessage>),
    WebSocket(WebSocketError),
    Json(serde_json::Error),
}

impl From<SendError<OwnedMessage>> for JoinError {
    fn from(e: SendError<OwnedMessage>) -> Self {
        return JoinError::SendError(e);
    }
}

impl From<serde_json::Error> for JoinError {
    fn from(e: serde_json::Error) -> Self {
        return JoinError::Json(e);
    }
}

impl From<WebSocketError> for JoinError {
    fn from(e: WebSocketError) -> Self {
        return JoinError::WebSocket(e);
    }
}

impl From<MessageError> for JoinError {
    fn from(error: MessageError) -> Self {
        match error {
            MessageError::SendError(e) => JoinError::SendError(e),
            MessageError::WebSocket(e) => JoinError::WebSocket(e),
            MessageError::Json(e) => JoinError::Json(e),
        }
    }
}
