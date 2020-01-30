use crate::browser::process_pipe::Process;
use crate::protocol;
use failure::{Fallible};
use log::{info, trace, warn};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::sync::{mpsc, Mutex};

#[cfg(unix)]
use std::os::unix::net::UnixStream;

#[cfg(unix)]
pub struct SocketConnection {
    sender: Mutex<BufWriter<UnixStream>>,
}

impl std::fmt::Debug for SocketConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "WebSocketConnection {{}}")
    }
}

impl SocketConnection {
    pub fn new(
        process: &Process,
        messages_tx: mpsc::Sender<protocol::Message>,
    ) -> Fallible<SocketConnection> {
        let receiver = BufReader::new(
            process
                .child_process
                .output
                .try_clone()
                .expect("Unable to clone input"),
        );

        std::thread::spawn(move || {
            trace!("Starting msg dispatching loop");
            Self::dispatch_incoming_messages(receiver, messages_tx);
            trace!("Quit loop msg dispatching loop");
        });
        let sender = BufWriter::new(
            process
                .child_process
                .input
                .try_clone()
                .expect("Unable to clone input"),
        );
        Ok(SocketConnection {
            sender: Mutex::new(sender),
        })
    }
    fn dispatch_incoming_messages(
        mut receiver: BufReader<UnixStream>,
        messages_tx: mpsc::Sender<protocol::Message>,
    ) {
        let mut message = vec![];
        loop {
            let delim = b"\0".last().unwrap();
            let read = receiver
                .read_until(delim.to_owned(), &mut message)
                .expect("Problem reading pipe");

            if read == 0 {
                break;
            }

            message.truncate(message.len() - 1);

            if let Ok(message_string) = std::str::from_utf8(&message) {
                if let Ok(message) = protocol::parse_raw_message(&message_string) {
                    if messages_tx.send(message).is_err() {
                        break;
                    }
                } else {
                    trace!(
                        "Incoming message isn't recognised as event or method response: {}",
                        message_string
                    );
                }
            } else {
                panic!("Got a weird message: {:?}", message)
            }
            message.clear();
        }
        info!("Sending shutdown message to message handling loop");
        if messages_tx
            .send(protocol::Message::ConnectionShutdown)
            .is_err()
        {
            warn!("Couldn't send message to transport loop telling it to shut down")
        }
    }
    pub fn send_message(&self, message_text: &str) -> Fallible<()> {
        let mut sender = self.sender.lock().unwrap();
        sender.write(message_text.as_bytes())?;
        sender.write(b"\0")?;
        sender.flush()?;
        Ok(())
    }
    // Todo
    pub fn shutdown(&self) {}
}

impl Drop for SocketConnection {
    fn drop(&mut self) {
        info!("dropping pipe connection");
    }
}
