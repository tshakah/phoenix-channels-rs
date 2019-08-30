use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};

use slog;
use slog_stdlog;
use slog::Drain;

use websocket::client::ClientBuilder;
use serde_json::value::Value;

use receiver::Receiver;
use sender::Sender;
use error::ConnectError;
use message::Message;
use error::{JoinError, MessageError};
use std::io::{Error, ErrorKind};
use event::EventKind;

use websocket::futures::sync::mpsc;
use websocket::futures::Future;
use websocket::futures::Stream;
use websocket::futures::Sink;

pub type MessageResult = Result<Message, MessageError>;

pub trait ClientSender {
    fn send(&mut self, topic: &str, event: EventKind, message: &Value);
}


const PHOENIX_VERSION: &str = "2.0.0";


#[derive(Debug)]
pub enum ClientError {
    Connect(ConnectError),
    Join(JoinError),
    Thread(String),
}

impl From<ConnectError> for ClientError {
    fn from(e: ConnectError) -> Self {
        return ClientError::Connect(e);
    }
}

impl From<JoinError> for ClientError {
    fn from(e: JoinError) -> Self {
        return ClientError::Join(e);
    }
}



pub fn connect(url: &str, params: Vec<(&str, &str)>, logger: Option<slog::Logger>) -> Result<(Sender, Receiver), ConnectError> {
    let logger = logger.unwrap_or(slog::Logger::root(slog_stdlog::StdLog.fuse(), o!()));

    // create a channel to send data to the socket
    let (to_socket, from_lib) = mpsc::channel(10);
    let (to_lib, from_socket) = mpsc::channel(10);

    // convert the params to a uri component string
    let mut params_uri: String = "".to_owned();
    for (k, v) in params {
        params_uri.push_str(&format!("&{}={}", k, v));
    }

    // create a phoenix socket url with params expanded and parse it
    // phoenix socket endpoints always have /websocket appended for the socket route
    // it also adds the vsn parameter for versioning
    let addr = format!("{}/websocket?vsn={}{}", url, PHOENIX_VERSION, params_uri);

    println!("{:?}", addr);

    let connection = ClientBuilder::new(&addr)
        .unwrap()
        .async_connect(None)
        .map(|(duplex, _)| duplex.split())
        .and_then(|(sink, stream)| {
            let sink_handle = thread::spawn(move || {
                sink
                    .send_all(from_lib.map_err(|_e| Error::new(ErrorKind::Other, "Sink receiver error")))
                    .map(|_| ()).wait().unwrap();
            });

            let stream_handle = thread::spawn(move || {
                stream
                    .for_each(|i| {
                        let sender = to_lib.clone();
                        let msg = Message::from_result(i).unwrap();

                        sender.send(msg).wait().unwrap();
                        Ok(())
                    }).wait().unwrap();
            });

            Ok((sink_handle, stream_handle))
        });

    let (sink_handle, stream_handle) = connection.wait()?;

    let sender = Sender::new(to_socket, sink_handle, logger.new(o!("type" => "sender")));
    let receiver = Receiver::new(from_socket, stream_handle, logger.new(o!("type" => "receiver")));

    return Ok((sender, receiver));
}


pub struct Client {
    logger: slog::Logger,
    sender_ref: Arc<Mutex<Sender>>,
    heartbeat_handle: thread::JoinHandle<()>,
}

impl Client {
    pub fn new(url: &str, params: Vec<(&str, &str)>, logger: Option<slog::Logger>) -> Result<(Client, mpsc::Receiver<Message>), ClientError> {
        let logger = logger.unwrap_or(slog::Logger::root(slog_stdlog::StdLog.fuse(), o!()));
        debug!(logger, "creating client"; "url" => url);

        let (sender, receiver) = connect(url, params, Some(logger.clone()))?;

        let sender_ref = Arc::new(Mutex::new(sender));
        let heartbeat = Client::keepalive(Arc::clone(&sender_ref));

        let client = Client {
            logger: logger,
            sender_ref: sender_ref,
            heartbeat_handle: heartbeat,
        };

        return Ok((client, receiver.reader));
    }

    fn keepalive(sender_ref: Arc<Mutex<Sender>>) -> thread::JoinHandle<()> {
        return thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(2));
                // if the mutex is poisoned then the whole thread wont work
                let mut sender = sender_ref.lock().unwrap();
                sender.heartbeat();
            }
        });
    }

    pub fn join(&self, channel: &str) -> Result<u32, ClientError> {
        return match self.sender_ref.lock() {
            Ok(mut sender) => Ok(sender.join(channel)?),
            Err(_) => Err(ClientError::Thread(String::from("Cannot join as sender mutex has been poisoned"))),
        };
    }

    pub fn join_threads(self) -> thread::Result<()> {
        self.heartbeat_handle.join()?;

        Ok(())
    }
}

impl ClientSender for Client {
    fn send(&mut self, topic: &str, event: EventKind, message: &Value) {
        let mut sender = self.sender_ref.lock().unwrap();
        sender.send(topic, event, message);
    }
}
