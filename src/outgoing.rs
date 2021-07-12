use websocket::{OwnedMessage, Message};
use std::sync::mpsc::Receiver;
use websocket::sender::Writer;
use std::net::TcpStream;

pub fn process_outgoing(websocket_sending_rx: &Receiver<OwnedMessage>, sender: &mut Writer<TcpStream>) {
    loop {
        // Send loop
        let message = match websocket_sending_rx.recv() {
            Ok(m) => m,
            Err(e) => {
                println!("Send Loop: {:?}", e);
                return;
            }
        };
        match message {
            OwnedMessage::Close(_) => {
                let _ = sender.send_message(&message);
                // If it's a close message, just send it and then return.
                return;
            }
            _ => (),
        }
        // Send the message
        // println!("sending out {:?}",message);
        match sender.send_message(&message) {
            Ok(()) => (),
            Err(e) => {
                println!("Send Loop: {:?}", e);
                let _ = sender.send_message(&Message::close());
                return;
            }
        }
    }

}
