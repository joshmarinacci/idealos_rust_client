use core::option::Option::None;
use core::result::Result::{Err, Ok};
use serde_json::error::Result;
use serde_json::value::Value;
use websocket::receiver::Reader;
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use websocket::OwnedMessage;
use crate::messages::{RenderMessage, CloseWindowScreen, window_list_message, group_message, any_graphics_message};
use idealos_schemas::windows::{WindowOpenDisplay_name, WindowOpenDisplay, create_child_window_display_name, create_child_window_display, close_child_window_display_name, close_child_window_display};
use idealos_schemas::graphics::*;
use idealos_schemas::general::{Connected_name};
use crate::messages::RenderMessage::Connected;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Error;
use sdl2::gfx::imagefilter::sub;


fn parse_message(renderloop_send:&Sender<RenderMessage>, txt:String) -> Result<()>{
    let v: Value = serde_json::from_str(txt.as_str())?;
    // println!("got a message: {:}",v);
    match &v["type"] {
        Value::String(msg_type) => {
            if msg_type == Connected_name {
                println!("really connected");
                return Ok(())
            }
            if msg_type == WindowOpenDisplay_name {
                let msg:WindowOpenDisplay = serde_json::from_str(txt.as_str())?;
                renderloop_send.send(RenderMessage::OpenWindow(msg));
                return Ok(())
            }
            if msg_type == "MAKE_window_list_name" {
                println!("Processed window list name");
                let msg: window_list_message = serde_json::from_str(txt.as_str())?;
                println!("the window list is {:?}",msg);
                renderloop_send.send(RenderMessage::WindowList(msg));
                return Ok(())
            }

            if msg_type == "group-message" {
                // println!("got a group message {:?}",txt);
                let msg: group_message = serde_json::from_str(txt.as_str())?;
                // println!("the group message is {:?}",msg);
                for(sub_mess) in &msg.messages {
                    // println!("sub message is {:?}",sub_mess);
                    match sub_mess {
                        any_graphics_message::MAKE_DrawRect_name { window, color, x, y, width, height } => {
                            renderloop_send.send(RenderMessage::FillRect(DrawRect{
                                type_: "".to_string(),
                                window: String::from(window),
                                color: color.to_string(),
                                x: *x,
                                y: *y,
                                width: *width,
                                height: *height
                            }));
                        }
                        any_graphics_message::MAKE_DrawImage_name { window, x, y, width, height, pixels } => {
                            renderloop_send.send(RenderMessage::DrawImage(DrawImage{
                                type_: "".to_string(),
                                window: window.to_string(),
                                x: *x,
                                y: *y,
                                width: *width,
                                height: *height,
                                pixels: pixels.to_vec()
                            }));
                        }
                    }
                }
                return Ok(())
            }

            if msg_type == create_child_window_display_name {
                let msg:create_child_window_display = serde_json::from_str(txt.as_str())?;
                renderloop_send.send(RenderMessage::CreateChildWindow(msg));
                return Ok(())
            }
            if msg_type == close_child_window_display_name {
                let msg:close_child_window_display = serde_json::from_str(txt.as_str())?;
                renderloop_send.send(RenderMessage::CloseChildWindow(msg));
                return Ok(())
            }
            if msg_type == DrawPixel_name {
                let msg:DrawPixel = serde_json::from_str(txt.as_str())?;
                renderloop_send.send(RenderMessage::DrawPixel(msg));
                return Ok(())
            }
            if msg_type == DrawRect_name {
                let msg:DrawRect = serde_json::from_str(txt.as_str())?;
                renderloop_send.send(RenderMessage::FillRect(msg));
                return Ok(())
            }
            if msg_type == DrawImage_name {
                let msg:DrawImage = serde_json::from_str(txt.as_str())?;
                renderloop_send.send(RenderMessage::DrawImage(msg));
                return Ok(())
            }
            match &msg_type[..] {
                "WINDOW_CLOSE" => {
                    let msg:CloseWindowScreen = serde_json::from_str(txt.as_str())?;
                    renderloop_send.send(RenderMessage::CloseWindow(msg));
                    ()
                },
                _ => {
                    println!("some other message type {}",txt)
                }
            }
        }
        _ => {
            println!("data that's not a message!!")
        }
    }
   Ok(())
}

pub fn process_incoming(receiver: &mut Reader<TcpStream>, websocket_sending_tx: &Sender<OwnedMessage>, render_loop_send: &Sender<RenderMessage>) {
    // Receive loop
    for message in receiver.incoming_messages() {
        //if error, send back a close message directly
        let message = match message {
            Ok(m) => m,
            Err(e) => {
                println!("Receive Loop: {:?}", e);
                let _ = websocket_sending_tx.send(OwnedMessage::Close(None));
                return;
            }
        };
        match message {
            OwnedMessage::Close(_) => {
                println!("got a close message");
                // Got a close message, so send a close message and return
                let _ = websocket_sending_tx.send(OwnedMessage::Close(None));
                return;
            }
            // Say what we received
            OwnedMessage::Text(txt) => {
                // println!("received message {:?}", txt);
                let res = parse_message(render_loop_send, txt);
                match res {
                    Ok(_) => { }
                    Err(err) => {
                        println!("error processing message {:?}",err)
                    }
                }
            }
            _ => {
                println!("Receive Loop: {:?}", message);
            },
        }
    }
}
