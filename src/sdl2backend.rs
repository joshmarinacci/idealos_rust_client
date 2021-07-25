use std::time::Duration;
use std::sync::mpsc::{Sender, Receiver};
use std::collections::HashMap;
use websocket::OwnedMessage;
use serde_json::{json};

use crate::window::{Window, Point, Insets, Bounds, Dimensions};
use crate::messages::{RenderMessage, MouseDown, MouseDown_name, MouseUp, MouseUp_name, set_focused_window_message, KeyboardDown, KeyboardDown_name, WindowSetPosition_message, WindowSetPosition, WindowSetSize, WindowSetSize_message};
use crate::fontinfo::FontInfo;


use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::{WindowCanvas, Texture, TextureCreator, Canvas, RenderTarget};
use sdl2::Sdl;
use crate::common::send_refresh_all_windows_request;
use sdl2::video::WindowContext;
use sdl2::rect::Rect;
use sdl2::mouse::{MouseButton, MouseState};
use colors_transform::{Rgb, Color as CTColor};
use crate::font::{FontInfo2, GlyphInfo};
use sdl2::surface::Surface;

const SCALE: u32 = 2;
const SCALEI: i32 = SCALE as i32;
const BORDER:Insets = Insets {
    left: 1,
    right: 1,
    top: 10,
    bottom: 1,
};
const RESIZE:Dimensions = Dimensions {
    width: 10,
    height: 10
};

pub struct SDL2Backend<'a> {
    pub active_window:Option<String>,
    pub sdl_context: &'a Sdl,
    pub canvas: WindowCanvas,
    pub creator: &'a TextureCreator<WindowContext>,
    pub window_buffers:HashMap<String,Texture<'a>>,
    pub window_order:Vec<String>,
    pub dragging:bool,
    pub resizing:bool,
    pub dragtarget:Option<String>,
    pub font_info:FontInfo2,
}


impl<'a> SDL2Backend<'a> {
    fn process_render_messages(&mut self,
                               windows:&mut HashMap<String, Window>,
                               input: &Receiver<RenderMessage>,
                               output: &Sender<OwnedMessage>,
    ) {
        'main: loop {
            match input.try_recv() {
                Ok(msg) => {
                    // println!("incoming message {:?}",msg);
                    match msg {
                        RenderMessage::OpenWindow(m) => {
                            println!("opening a window {:?}",m);
                            let win:Window = Window {
                                id: m.window.id.clone(),
                                x: m.window.x as i32,
                                y: m.window.y as i32,
                                width: m.window.width as i32,
                                height: m.window.height as i32,
                                owner: m.window.owner.clone(),
                                window_type: m.window.window_type.clone(),
                                title: "title".to_string()
                            };
                            self.init_window(&win);
                            // self.window_buffers.insert(win.id.clone(),win);
                            windows.insert(m.window.id.clone(), win);
                            // println!("window count is {}", windows.len());
                        }
                        RenderMessage::WindowSetSize(m) => {
                            if let Some(win) = windows.get_mut(m.window.as_str()) {
                                win.width = m.width as i32;
                                win.height = m.height as i32;
                                self.resize_window(win);
                            }
                        }
                        RenderMessage::CreateChildWindow(m) => {
                            // println!("creating a child window");
                            if let Some(win) = windows.get_mut(&m.parent) {
                                let child:Window = Window {
                                    id:m.window.id.clone(),
                                    x: m.window.x as i32,
                                    y: m.window.y as i32,
                                    width: m.window.width as i32,
                                    height: m.window.height as i32,
                                    owner: m.window.owner.clone(),
                                    window_type: m.window.window_type.clone(),
                                    title: "title".to_string()
                                };

                                self.init_window(&child);
                                windows.insert(child.id.clone(),  child);
                            }
                        }
                        RenderMessage::CloseChildWindow(m) => {
                            if let Some(win) = windows.get_mut(m.window.as_str()) {
                                self.close_window(win);
                                windows.remove(m.window.as_str());
                            }
                        }
                        RenderMessage::WindowList(m) => {
                            // println!("window list");
                            for (key, value) in &m.windows {
                                // println!("make window id {} at {},{}", value.id, value.x, value.y);
                                let win = Window::from_info2(&value);
                                self.init_window(&win);
                                windows.insert(win.id.clone(), win);
                            }
                            println!("window count is {:?}", windows.len());
                            send_refresh_all_windows_request(&windows, &output);
                        },
                        RenderMessage::CloseWindow(m) => {
                            // println!("closing a window {:?}",m);
                            if let Some(win) = windows.get_mut(m.window.id.as_str()) {
                                self.close_window(win);
                                windows.remove(m.window.id.as_str());
                            }
                        },
                        RenderMessage::DrawPixel(m) => {
                            if let Some(win) = windows.get_mut(m.window.as_str()) {
                                if let Some(tex) = self.window_buffers.get_mut(win.id.as_str()) {
                                    self.canvas.with_texture_canvas(tex, |texture_canvas| {
                                        texture_canvas.set_draw_color(lookup_color(&m.color));
                                        texture_canvas
                                            .fill_rect(Rect::new(m.x as i32,
                                                                 m.y as i32, 1, 1))
                                            .expect("could not fill rect");
                                        // println!("drew pixel to texture at {},{} c={}",m.x,m.y, m.color);
                                    });
                                }
                            }
                        },
                        RenderMessage::FillRect(m) => {
                            // println!("fill rect {:?}",m);
                            if let Some(win) = windows.get_mut(m.window.as_str()) {
                                if let Some(tex) = self.window_buffers.get_mut(win.id.as_str()) {
                                    self.canvas.with_texture_canvas(tex, |texture_canvas| {
                                        texture_canvas.set_draw_color(lookup_color(&m.color));
                                        texture_canvas
                                            .fill_rect(Rect::new(m.x as i32, m.y as i32, m.width as u32, m.height as u32))
                                            .expect("could not fill rect");
                                        // println!("drew rect to texture at {},{} - {}x{}",m.x,m.y,m.width,m.height);
                                    });
                                }
                            }
                        }
                        RenderMessage::DrawImage(m) => {
                            if let Some(win) = windows.get_mut(m.window.as_str()) {
                                if let Some(tex) = self.window_buffers.get_mut(win.id.as_str()) {
                                    // println!("drawing an image {}x{}", m.width, m.height);
                                    self.canvas.with_texture_canvas(tex,|texture_canvas|{
                                        // println!("drew image to texture at {},{} - {}x{}, count={}",m.x,m.y,m.width,m.height,m.pixels.len());
                                        for i in 0..m.width {
                                            for j in 0..m.height {
                                                let n:usize = ((j * m.width + i) * 4) as usize;
                                                let alpha = m.pixels[n+3];
                                                if m.depth == 8 {
                                                    //if 8bit depth then it's a real RGBA image
                                                    if alpha > 0 {
                                                        let col = Color::RGBA(m.pixels[n + 0], m.pixels[n + 1], m.pixels[n + 2], m.pixels[n + 3]);
                                                        texture_canvas.set_draw_color(col);
                                                        texture_canvas.fill_rect(Rect::new((m.x + i) as i32, (m.y + j) as i32, 1, 1));
                                                    }
                                                } else if m.depth == 1 {
                                                    //if 1bit depth and a color is set, then draw with that color wherever not transparent (alpha > 0)
                                                    if alpha > 0 {
                                                        let col = lookup_color(&m.color);
                                                        texture_canvas.set_draw_color(col);
                                                        texture_canvas.fill_rect(Rect::new((m.x + i) as i32, (m.y + j) as i32, 1, 1));
                                                    }
                                                    //else assume it's just black wherever not transparent (alpha > 0)
                                                } else {
                                                    if alpha > 0 {
                                                        let col = Color::RGBA(m.pixels[n + 0], m.pixels[n + 1], m.pixels[n + 2], m.pixels[n + 3]);
                                                        texture_canvas.set_draw_color(col);
                                                        texture_canvas.fill_rect(Rect::new((m.x + i) as i32, (m.y + j) as i32, 1, 1));
                                                    }
                                                }
                                            }
                                        }
                                    });
                                }
                            }
                        }
                        _ => {
                            println!("unhandled message {:?}",msg);
                        }
                    }
                }
                Err(e) => break
            }
        }
    }
    fn init_window(&mut self, win: &Window) {
        let mut tex = self.creator.create_texture_target(PixelFormatEnum::RGBA8888,
                                                         win.width as u32,
                                                         win.height as u32
                                                         // 256,256
        )
            .map_err(|e|e.to_string()).unwrap();
        println!("made texture {}x{}",win.width, win.height);
        self.canvas.with_texture_canvas(&mut tex, |tc|{
            tc.clear();
            tc.set_draw_color(Color::RGBA(0,0,0,255));
            tc
                .fill_rect(Rect::new(0, 0, (win.width as u32) as u32, (win.height as u32) as u32));
        });
        self.window_buffers.insert(win.id.clone(),tex);
        self.window_order.push(win.id.clone());
    }
    fn resize_window(&mut self, win: &Window) {
        self.window_buffers.remove(win.id.as_str());
        let mut tex = self.creator.create_texture_target(PixelFormatEnum::RGBA8888,win.width as u32, win.height as u32)
            .map_err(|e|e.to_string()).unwrap();
        self.canvas.with_texture_canvas(&mut tex, |tc|{
            tc.clear();
            tc.set_draw_color(Color::RGBA(0,0,0,255));
            tc
                .fill_rect(Rect::new(0, 0, (win.width as u32) as u32, (win.height as u32) as u32));
        });
        self.window_buffers.insert(win.id.clone(),tex);
    }
    fn close_window(&mut self, win: &mut Window) {
        // println!("found texture for window");
        //destroy the texture
        //remove from window_buffers
        self.window_buffers.remove(win.id.as_str());
        if let Some(n) = self.window_order.iter().position(|id|id == &win.id) {
            self.window_order.remove(n);
        }
    }
    pub fn start_loop(&mut self,
                      windows: &mut HashMap<String, Window>,
                      input: &Receiver<RenderMessage>,
                      output: &Sender<OwnedMessage>
        ) -> Result<(),String> {
        println!("sdl2 backend");

        let mut event_pump = self.sdl_context.event_pump()?;

        'done:loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => {
                        println!("quitting");
                        break 'done;
                    },
                    Event::KeyDown {keycode,keymod,..} => self.process_keydown(keycode, keymod, windows,output),
                    Event::MouseButtonDown { x, y,mouse_btn, .. } => self.process_mousedown(x,y,mouse_btn, windows, output),
                    Event::MouseButtonUp {x,y,mouse_btn,..} =>  self.process_mouseup(x,y,mouse_btn,windows,output),
                    _ => {}
                }
            }
            self.process_mousedrag(&event_pump.mouse_state(), windows);

            self.process_render_messages(windows,
                                         input,
                                         output,
            );
            self.draw_windows(windows);
            self.draw_cursor(&event_pump.mouse_state());
            self.canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
        println!("SDL thread is ending");

        Ok(())
    }

    fn draw_windows(&mut self, windows: &mut HashMap<String, Window>) {
        self.canvas.set_draw_color(Color::RGBA(255,0,255,255));
        self.canvas.clear();
        //clear background to white
        //for each window
        for id in self.window_order.iter() {
            if let Some(win) = windows.get(id) {
                if let Some(tex) = self.window_buffers.get(id) {
                    //draw background / border
                    // println!("drawing window type {:?}",win.window_type);
                    match win.window_type.as_str() {
                        "MENUBAR" => {}
                        "DOCK" => {}
                        "SIDEBAR" => {}
                        "CHILD" => {}
                        "PLAIN" => {
                            // self.canvas.set_draw_color(self.calc_window_border_color(win));
                            self.canvas.set_draw_color(Color::RED);
                            self.canvas.fill_rect(Rect::new(
                                ((win.x-BORDER.left)*(SCALE as i32)) as i32,
                                ((win.y-BORDER.top)*(SCALE as i32)) as i32,
                                (BORDER.left+win.width+BORDER.right)as u32*SCALE as u32,
                                (BORDER.top+win.height+BORDER.bottom)as u32*SCALE as u32));
                            draw_title(&mut self.canvas, &self.font_info, &win)
                        }
                        _ => {
                            println!("unknown window type {:?}",win.window_type);
                        }
                    }
                    //draw window texture
                    let dst = Some(Rect::new((win.x as u32*SCALE) as i32,
                                             (win.y as u32*SCALE) as i32,
                                             (win.width as u32 * SCALE as u32) as u32,
                                             (win.height as u32 * SCALE as u32) as u32
                    ));
                    self.canvas.copy(tex,None,dst);
                }
            }
        }
        // self.font.draw_text_at("idealos", 150,0,&Color::GREEN, &mut self.canvas, SCALEI);
    }
    fn process_keydown(&self, keycode: Option<Keycode>,  keymod:Mod, windows:&mut HashMap<String,Window>, output: &Sender<OwnedMessage>) {
        if let Some(keycode) = keycode {
            match keycode {
                Keycode => {
                    if let Some(id) = &self.active_window {
                        if let Some(win) = windows.get_mut(id) {
                            println!("got a message {:?}",keycode.name());
                            println!("mod is {:?}",keymod);
                            let mut key = keycode.name().to_lowercase();
                            let mut name = keycode.name();
                            let shift = (keymod == Mod::LSHIFTMOD || keymod == Mod::RSHIFTMOD);
                            if(shift) {
                                key = keycode.name().to_uppercase();
                            }
                            let control = (keymod == Mod::LCTRLMOD || keymod == Mod::RCTRLMOD);
                            let mut code = format!("{}{}","Key",name);
                            if keycode.name().eq("Left") { code = "ArrowLeft".to_string(); }
                            if keycode.name().eq("Right") { code = "ArrowRight".to_string(); }
                            if keycode.name().eq("Up") { code = "ArrowUp".to_string(); }
                            if keycode.name().eq("Down") { code = "ArrowDown".to_string(); }
                            if keycode.name().eq("Backspace") { code = "Backspace".to_string(); }
                            if keycode.name().eq("Space") {
                                code = "Space".to_string();
                                key = " ".to_string();
                            }
                            println!("code is {} key is {}",code, key);
                            //keycode.name is Left
                            let msg = KeyboardDown {
                                type_: KeyboardDown_name.to_string(),
                                code: code,
                                key:key,
                                shift:shift,
                                alt:false,
                                meta:false,
                                control,
                                app:win.owner.to_string(),
                                target: win.owner.clone(),
                                window: win.id.to_string()
                            };
                            output.send(OwnedMessage::Text(json!(msg).to_string()));
                        }
                    }
                }
                _ => {
                }
            }
        }

    }
    fn process_mousedown(&mut self, x: i32, y: i32, mouse_btn: MouseButton, windows: &mut HashMap<String, Window>, output: &Sender<OwnedMessage>) {
        match mouse_btn {
            MouseButton::Left => {
                let pt = Point { x: x / SCALE as i32, y: y / SCALE as i32, };
                for win in windows.values() {
                    if win.resize_contains(&pt, &RESIZE) {
                        self.resizing = true;
                        self.dragtarget = Some(win.id.clone());
                        break;
                    }
                    if win.contains(&pt) {
                        if win.window_type.eq("PLAIN") {
                            self.active_window = Some(win.id.clone());
                            let window_focus_msg = set_focused_window_message {
                                type_: "MAKE_SetFocusedWindow_name".to_string(),
                                window: win.id.to_string()
                            };
                            output.send(OwnedMessage::Text(json!(window_focus_msg).to_string()));
                            self.raise_window(win);
                        }
                        let msg = MouseDown {
                            type_:MouseDown_name.to_string(),
                            x: ((pt.x) - win.x) as i64,
                            y: ((pt.y) - win.y) as i64,
                            target: win.owner.clone(),
                            window: win.id.to_string(),
                        };
                        output.send(OwnedMessage::Text(json!(msg).to_string()));
                        continue;
                    }
                    if win.border_contains(&pt, &BORDER) {
                        // println!("clicked on the border");
                        self.dragging = true;
                        self.dragtarget = Some(win.id.clone());
                    }
                }
            }
            _ => {}
        };

    }
    fn process_mouseup(&mut self, x: i32, y: i32, mouse_btn: MouseButton, windows: &mut HashMap<String, Window>, output: &Sender<OwnedMessage>) {
        if self.dragging {
            if let Some(winid) = &self.dragtarget {
                if let Some(win) = windows.get(winid) {
                    let pt = Point { x: x / SCALE as i32, y: y / SCALE as i32, };
                    let move_msg = WindowSetPosition {
                        type_: WindowSetPosition_message.to_string(),
                        app: String::from("someappid"),
                        window: winid.to_string(),
                        x: pt.x as i64,
                        y: pt.y as i64,
                    };
                    // println!("setting window position {:?}",move_msg);
                    output.send(OwnedMessage::Text(json!(move_msg).to_string()));
                }
            }
            self.dragging = false;
        }

        if self.resizing {
            if let Some(winid) = &self.dragtarget {
                if let Some(win) = windows.get(winid) {
                    let edge = Point { x: x / SCALE as i32, y: y / SCALE as i32, };
                    let pos = Point { x: win.x, y: win.y };
                    let size_msg = WindowSetSize {
                        type_: WindowSetSize_message.to_string(),
                        app: String::from("someappid"),
                        window: winid.to_string(),
                        width: (edge.x - pos.x) as i64,
                        height: (edge.y - pos.y)as i64,
                    };
                    output.send(OwnedMessage::Text(json!(size_msg).to_string()));

                    self.resize_window(win);
                }
            }
            self.resizing = false;
        }

        if let MouseButton::Left = mouse_btn {
            let pt = Point { x: x / SCALE as i32, y: y / SCALE as i32, };
            if let Some(id) = &self.active_window {
                if let Some(win) = windows.get(id) {
                    let msg = MouseUp {
                        type_: MouseUp_name.to_string(),
                        x: ((pt.x) - win.x) as i64,
                        y: ((pt.y) - win.y) as i64,
                        target: win.owner.clone(),
                        window: win.id.to_string(),
                    };
                    output.send(OwnedMessage::Text(json!(msg).to_string()));
                }
            }
        }

    }
    fn calc_window_border_color(&self, win: &Window) -> Color {
        return if self.active_window == Some(win.id.clone()) {
            Color::RGBA(0, 255, 255, 255)
        } else {
            Color::RGBA(255, 255, 0, 255)
        }
    }
    fn process_mousedrag(&self, mouse_state:&MouseState, windows:&mut HashMap<String,Window>) -> () {
        if self.dragging {
            if let Some(winid) = &self.dragtarget {
                if let Some(win) = windows.get_mut(winid) {
                    // println!("dragging {} {} with {:?}", mouse_state.x(), mouse_state.y(), win.id);
                    win.x = mouse_state.x()/SCALEI;
                    win.y = mouse_state.y()/SCALEI;
                }
            }
        }
        if self.resizing {
            if let Some(winid) = &self.dragtarget {
                if let Some(win) = windows.get_mut(winid) {
                    win.width = (mouse_state.x()/SCALEI) - win.x;
                    win.height = (mouse_state.y()/SCALEI) - win.y;
                }
            }
        }

    }
    fn raise_window(&mut self, win: &Window) {
        if let Some(n) = self.window_order.iter().position(|x|x == &win.id) {
            let id = self.window_order.remove(n);
            self.window_order.push(id)
        }
    }
    fn draw_cursor(&mut self, mouse: &MouseState) {
        if let Some(cursor_glyph) = lookup_char(&self.font_info, 1) {
            draw_glyph(&mut self.canvas, cursor_glyph, mouse.x()/SCALEI, mouse.y()/SCALEI);
        }
    }
}

fn lookup_color(name: &String) -> Color {
    if name.starts_with("#") {
        // println!("its hex");
        let col = Rgb::from_hex_str(name).unwrap();
        // println!("parsed the color ${:?}",col);
        return Color::RGBA(col.get_red() as u8, col.get_green() as u8, col.get_blue() as u8, 255);
    }
    return match name.as_str() {
        "red" => Color::RED,
        "black" => Color::BLACK,
        "blue" => Color::BLUE,
        "white" => Color::WHITE,
        "green" => Color::GREEN,
        "yellow" => Color::YELLOW,
        "grey" => Color::GREY,
        "gray" => Color::GRAY,
        "magenta" => Color::MAGENTA,
        "teal" => Color::RGB(0,128,128),
        "aqua" => Color::RGB(0,255,255),
        "cyan" => Color::CYAN,
        _ => {
            println!("unknown color {}",name);
            Color::MAGENTA
        },
    }
}

pub fn draw_title(canvas:&mut WindowCanvas, font:&FontInfo2, win:&Window) {
    let mut ww:i32 = 0;
    for ch in win.title.bytes() {
        let glyph_opt = lookup_char(font,ch);
        if let Some(glyph) = glyph_opt {
            canvas.set_draw_color(Color::RED);
            draw_glyph(canvas,glyph,win.x- glyph.left +ww,win.y-BORDER.top);
            ww += (glyph.width - glyph.left - glyph.right) as i32;
            ww += 1;
        }
    }
}

pub fn draw_glyph(canvas:&mut WindowCanvas, glyph: &GlyphInfo, x: i32, y: i32) {
    let w:i32 = glyph.width as i32;
    let h:i32 = glyph.height as i32;
    let f = 1;
    for i in glyph.left .. glyph.width - glyph.right as i32 {
        for j in 0 .. glyph.height {
            let n:usize = (j * w + i) as usize;
            let alpha = glyph.data[n];
            if alpha > 0 {
                canvas.set_draw_color(Color::BLACK);
                canvas.fill_rect(Rect::new(
                    (i + x)*SCALEI as i32,
                    (y + j + f)*SCALEI as i32,
                    SCALE, SCALE
                ));
            }
        }
    }
}

pub fn lookup_char(p0: &FontInfo2, ch:u8) -> Option<& GlyphInfo> {
    return p0.glyphs.iter().find(|g| {
        return g.id == (ch as u32)
    })
}
