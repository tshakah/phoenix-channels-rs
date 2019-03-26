use slog;
use message::Message;
use websocket::futures::sync::mpsc;
use std::thread::JoinHandle;

pub struct Receiver
{
    logger: slog::Logger,
    stream_handle: JoinHandle<()>,
    pub reader: mpsc::Receiver<Message>,
}

impl Receiver {
    pub fn new(reader: mpsc::Receiver<Message>, stream_handle: JoinHandle<()>, logger: slog::Logger) -> Receiver {
        Receiver {
            logger: logger,
            stream_handle: stream_handle,
            reader: reader,
        }
    }
}
