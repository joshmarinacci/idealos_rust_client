use crate::window::Window;
use std::collections::HashMap;
use crate::messages::RenderMessage;
use std::sync::mpsc::{Receiver, Sender};
use websocket::OwnedMessage;

pub trait Backend {
    fn start_loop(&mut self, windows: &mut HashMap<String, Window>, incoming: &Receiver<RenderMessage>, outgoing:&Sender<OwnedMessage>) -> Result<(),String>;

    }