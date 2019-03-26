use slog;
use serde_json;

use websocket::OwnedMessage;
use websocket::futures::sync::mpsc::Sender as Writer;
use websocket::futures::Sink;
use websocket::futures::Future;
use serde_json::value::Value;
use std::thread::JoinHandle;

use error::{JoinError, MessageError};
use event::EventKind;

pub struct Sender
{
    logger: slog::Logger,
    writer: Writer<OwnedMessage>,
    sink_handle: JoinHandle<()>,
    join_ref: u32,
    message_ref: u32,
}

impl Sender {
    pub fn new(writer: Writer<OwnedMessage>, sink_handle: JoinHandle<()>, logger: slog::Logger) -> Sender {
        Sender {
            logger: logger,
            writer: writer,
            sink_handle: sink_handle,
            join_ref: 0,
            message_ref: 0,
        }
    }

    pub fn join(&mut self, channel: &str) -> Result<u32, JoinError> {
        let phx_message = json!([self.join_ref, self.message_ref, channel, "phx_join", {}]);

        self.join_ref += 1;

        // serialise the message and use it to join the channel
        let serialised = serde_json::to_string(&phx_message)?;
        debug!(self.logger, "join()"; "payload" => &serialised);
        let message = OwnedMessage::Text(serialised);

        self.actually_send(message)?;

        return Ok(self.join_ref);
    }

    pub fn heartbeat(&mut self) -> Result<(u32), MessageError> {
        let phx_message = json!([(), self.message_ref, "phoenix", "heartbeat", {}]);

        let serialised = serde_json::to_string(&phx_message)?;
        debug!(self.logger, "heartbeat()"; "payload" => &serialised);
        let message = OwnedMessage::Text(serialised);

        self.actually_send(message)
    }

    pub fn send(&mut self, topic: &str, event: EventKind, message: &Value) -> Result<(u32), MessageError> {
        let phx_message = json!([(), self.message_ref, topic, event, message]);

        let serialised = serde_json::to_string(&phx_message)?;
        debug!(self.logger, "sent()"; "payload" => &serialised);
        let message = OwnedMessage::Text(serialised);

        self.actually_send(message)
    }

    fn actually_send(&mut self, message: OwnedMessage) -> Result<(u32), MessageError> {
        self.message_ref += 1;

        let writer = self.writer.clone();
        writer.send(message).wait()?;

        Ok(self.message_ref)
    }
}
