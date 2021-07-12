use std::collections::HashMap;

use crate::backend::Backend;

use crate::window::{Window, Point};
use websocket::OwnedMessage;
use crate::messages::{MouseDownMessage, MouseUpMessage, RenderMessage};
use std::sync::mpsc::{Sender, Receiver};
use serde_json::{json};

/*
use raylib::{RaylibHandle, RaylibThread};
use raylib::core::drawing::*;
use raylib::ffi::MouseButton;
use raylib::color::Color;
use raylib::prelude::Image;
use raylib::core::math::{Rectangle, rvec2};
*/
use raylib::prelude::*;
use crate::common::send_refresh_all_windows_request;

pub struct RaylibBackend {
    rl:RaylibHandle,
    thread:RaylibThread,
    colors:HashMap<String,Color>,
    active_window:Option<String>,
    window_buffers:HashMap<String,RenderTexture2D>,
}

impl RaylibBackend {
    pub fn make(width:i32,height:i32,fps:u32) -> RaylibBackend {
        let mut colors:HashMap<String,Color> = HashMap::new();
        colors.insert("black".parse().unwrap(), Color::BLACK);
        colors.insert("white".parse().unwrap(), Color::WHITE);
        colors.insert("red".parse().unwrap(), Color::RED);
        colors.insert("green".parse().unwrap(), Color::GREEN);
        colors.insert("skyblue".parse().unwrap(), Color::SKYBLUE);
        colors.insert("blue".parse().unwrap(), Color::BLUE);

        // open window
        let (mut rl, thread) = raylib::init()
            .size(width, height)
            .title("Raylib Window")
            .build();
        rl.set_target_fps(fps);


        return RaylibBackend {
            rl:rl,
            thread:thread,
            colors:colors,
            active_window:Option::None,
            window_buffers: Default::default()
        }
    }

    fn process_render_drawing(&mut self, windows:&mut HashMap<String,Window>) {
        let mut d = self.rl.begin_drawing(&self.thread);
        d.clear_background(Color::WHITE);

        // println!("window count is {:?}",windows.len());

        for(_, win) in windows {
            //draw bg of window
            d.draw_rectangle(
                win.x* SCALE -1* SCALE,
                win.y* SCALE -1* SCALE,
                win.width* SCALE +2* SCALE,
                win.height* SCALE +2* SCALE,
                calc_window_background(win, &self.active_window),
            );
            //draw window buffer
            let src = Rectangle{
                x: 0.0,
                y: 0.0,
                width: win.width as f32,
                height: -win.height as f32,
            };
            let dst = Rectangle {
                x: (win.x* SCALE) as f32,
                y: (win.y* SCALE) as f32,
                width: (win.width* SCALE) as f32 ,
                height: (win.height* SCALE) as f32,
            };
            let origin = rvec2(0,0);
            if let Some(tex) = self.window_buffers.get_mut(&*win.id) {
                d.draw_texture_pro(&tex, src, dst, origin, 0.0, Color::WHITE)
            }
        }

    }

    fn init_window(&mut self, win:&Window) {
        let mut rl = &mut self.rl;
        let thread = &self.thread;
        let mut target = rl.load_render_texture(thread, win.width as u32, win.height as u32).unwrap();
        {
            let mut d = rl.begin_texture_mode(thread,  &mut target);
            d.clear_background(Color::MAROON);
            d.draw_circle(win.width/2,win.height/2,4.0,Color::GREEN);
        }
        self.window_buffers.insert(win.id.clone(),target);
    }

    fn process_render_messages(&mut self,
                               windows:&mut HashMap<String,Window>,
                               render_loop_receive:&Receiver<RenderMessage>,
                               tx:&Sender<OwnedMessage>,
    ) {
        //inside_func(backend:&mut Backend) {
        //backend.rl.cool_func() // cool func doesn't exist
        //let mut rl = &mut backend.rl
        //rl.cool_func() // now cool func exists
        //}

        let mut rl = &mut self.rl;
        let thread = &self.thread;
        let colors = &mut self.colors;
        loop {
            match render_loop_receive.try_recv() {
                Ok(msg) => {
                    // println!("the text is {:?}", msg);
                    // println!("got render message {:?}",msg);
                    match msg {
                        RenderMessage::OpenWindow(m) => {
                            println!("opening a window");
                            let win = Window::from_info(&m.window);
                            self.init_window(&win);
                            windows.insert(m.window.id.clone(), win);
                            println!("window count is {}", windows.len());
                            ()
                        }
                        RenderMessage::CloseWindow(m) => {
                            println!("closing a window");
                            windows.remove(m.window.id.as_str());
                            println!("window count is {}", windows.len());
                            ()
                        }
                        RenderMessage::WindowList(m) => {
                            for (key, value) in &m.windows {
                                println!("make window id {} at {},{}", value.id, value.x, value.y);
                                windows.insert(key.clone(), Window::from_info(&value));
                            }
                            println!("window count is {:?}", windows.len());
                            for(_, win) in windows.into_iter() {
                                self.init_window(win);
                            }

                            send_refresh_all_windows_request(&windows, &tx);
                        },
                        RenderMessage::DrawPixel(m) => {
                            match windows.get_mut(m.window.as_str()) {
                                None => {
                                    println!("no window found for {}", m.window.as_str())
                                }
                                Some(win) => {
                                    if let Some(color) = colors.get(m.color.as_str()) {
                                        if let Some(tex) = self.window_buffers.get_mut(&*win.id) {
                                            let mut d = rl.begin_texture_mode(thread, tex);
                                            d.draw_rectangle(m.x, m.y, 1, 1, color);
                                        }
                                    } else {
                                        println!("invalid color {}",m.color);
                                    }
                                }
                            }
                        },
                        RenderMessage::DrawImage(m) => {
                            match windows.get_mut(m.window.as_str()) {
                                None => {
                                    println!("no window found for {}", m.window.as_str())
                                }
                                Some(win) => {
                                    // println!("drawing an image {}x{} at {},{}, len = {}",m.width,m.height,m.x,m.y,m.pixels.len());
                                    let mut img = Image::gen_image_color(m.width, m.height, Color::WHITE);
                                    for i in 0..m.width {
                                        for j in 0..m.height {
                                            let n = ((j*m.width+i)*4) as usize;
                                            let r = m.pixels[n+0] as u8;
                                            let g = m.pixels[n+1] as u8;
                                            let b = m.pixels[n+2] as u8;
                                            let a = m.pixels[n+3] as u8;
                                            let col = Color::from((r,g,b,a));
                                            // println!("at {},{} color is {:?}",i,j,col);
                                            img.draw_pixel(i,j,col);
                                        }
                                    }
                                    if let Some(tex2) = self.window_buffers.get_mut(&*win.id) {
                                        let tex = rl.load_texture_from_image(thread, &img).unwrap();
                                        let mut d = rl.begin_texture_mode(thread, tex2);
                                        d.draw_texture(&tex, m.x, m.y, Color::WHITE);
                                    }
                                }
                            }
                        },
                        RenderMessage::FillRect(m) => {
                            match windows.get_mut(m.window.as_str()) {
                                None => {
                                    println!("no window found for {}", m.window.as_str())
                                }
                                Some(win) => {
                                    if let Some(color) = colors.get(m.color.as_str()) {
                                        // println!("drawing a rect {}x{} at {},{}",m.width,m.height,m.x,m.y);
                                        if let Some(tex2) = self.window_buffers.get_mut(&*win.id) {
                                            let mut d = rl.begin_texture_mode(thread, tex2);
                                            d.draw_rectangle(m.x, m.y, m.width, m.height, color);
                                        }
                                    } else {
                                        println!("invalid color {}",m.color);
                                    }
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    // println!("nothing ready");
                    break;
                }
            }
        }

    }

    fn process_keyboard_input(&mut self, windows:&HashMap<String,Window>, websocket_sender:&Sender<OwnedMessage>) {
        let pressed_key = self.rl.get_key_pressed();
        if let Some(pressed_key) = pressed_key {
            // println!("pressed key {:?}",pressed_key);
            // Certain keyboards may have keys raylib does not expect. Uncomment this line if so.
            // let pressed_key: u32 = unsafe { std::mem::transmute(pressed_key) };
            //d.draw_text(&format!("{:?}", pressed_key), 100, 12, 10, Color::BLACK);
        }
        if self.rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            // println!("right is down")
        }

    }

    fn process_mouse_input(&mut self, windows:&HashMap<String,Window>, websocket_sender:&Sender<OwnedMessage>) {
        // println!("mouse position {:?}",rl.get_mouse_position());
        // println!("button pressed {:?}",rl.is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON));
        if self.rl.is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON) {
            let pos = self.rl.get_mouse_position();
            let pt = Point {
                x:(pos.x/ SCALE as f32) as i32,
                y:(pos.y/ SCALE as f32) as i32,
            };

            for win in windows.values() {
                if win.contains(&pt) {
                    //win.set_active_window()
                    self.active_window = Some(win.id.clone());
                    let msg = MouseDownMessage {
                        type_:"MOUSE_DOWN".to_string(),
                        x:(pt.x)-win.x,
                        y:(pt.y)-win.y,
                        target:win.owner.clone()
                    };
                    // println!("sending mouse down {:?}",msg);
                    websocket_sender.send(OwnedMessage::Text(json!(msg).to_string()));
                }
            }
        }

        if self.rl.is_mouse_button_released(MouseButton::MOUSE_LEFT_BUTTON) {
            let pos = self.rl.get_mouse_position();
            let pt = Point {
                x:(pos.x/ SCALE as f32) as i32,
                y:(pos.y/ SCALE as f32) as i32,
            };

            for win in windows.values() {
                if win.contains(&pt) {
                    let msg = MouseUpMessage {
                        type_:"MOUSE_UP".to_string(),
                        x:(pt.x)-win.x,
                        y:(pt.y)-win.y,
                        target:win.owner.clone()
                    };
                    websocket_sender.send(OwnedMessage::Text(json!(msg).to_string()));
                }
            }
        }
    }

}


const SCALE: i32 = 10;

impl Backend<'_> for RaylibBackend {
    fn start_loop(&mut self, windows:&mut HashMap<String,Window>, input: &Receiver<RenderMessage>, output:&Sender<OwnedMessage>) -> Result<(),String> {
        while !self.rl.window_should_close() {
            // println!("SCALE is {:?}",&rl.get_window_scale_dpi());
            self.process_render_messages(windows, input, output);
            self.process_keyboard_input(windows,output);
            self.process_mouse_input(windows, output);
            self.process_render_drawing(windows);
        }

        Ok(())
    }
}

fn calc_window_background(win: &Window, active_window: &Option<String>) -> Color {
    match active_window {
        Some(id) => {
            if id.eq(&win.id) {
                Color::SKYBLUE
            } else {
                Color::BLACK
            }
        },
        None => Color::BLACK
    }
}

