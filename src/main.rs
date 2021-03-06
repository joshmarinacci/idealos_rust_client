use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::{thread, env};

use websocket::{OwnedMessage};
use websocket::ClientBuilder;

use messages::{RenderMessage};
use window::{Window};

use crate::incoming::process_incoming;
use crate::outgoing::process_outgoing;
use crate::sdl2backend::SDL2Backend;
use crate::fontinfo::FontInfo;
use std::fs::File;
use std::io::BufReader;
use std::error::Error;
use sdl2::pixels::PixelFormatEnum;
use image::io::Reader as ImageReader;
use sdl2::render::{BlendMode, TextureCreator, Texture};
use image::RgbaImage;
use sdl2::video::{WindowContext, DriverIterator, drivers};
use sdl2::mouse::Cursor;
use sdl2::surface::Surface;
use sdl2::rect::Rect;
use serde_json::{json};
use idealos_schemas::general::{ScreenStart_name, ScreenStart};
use crate::font::load_font2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::{VideoSubsystem, video, render, version};
use std::time::Duration;
use crate::messages::{SetScreenSize, SetScreenSize_message};
use structopt::StructOpt;

mod messages;
mod window;
mod incoming;
mod outgoing;
mod backend;
mod sdl2backend;
mod common;
mod fontinfo;
mod font;

#[derive(StructOpt)]
#[structopt(name = "example", about="example rust client usage")]
struct Cli {
    server:String,
    #[structopt(short, long)]
    fullscreen:bool,
    #[structopt(long=("--scale"),default_value="1")]
    scale: u32,
    #[structopt(long=("--width"),default_value="0")]
    width:u32,
    #[structopt(long=("--height"),default_value="0")]
    height:u32,
}
pub fn main() -> Result<(),String> {
    let args:Cli = Cli::from_args();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    println!("verison is {}", sdl2::version::version());
    println!("current driver is {:}",video_subsystem.current_video_driver());
    let display_count = video_subsystem.num_video_displays()?;
    println!("display count {:}",display_count);
    for d in drivers() {
        println!("video driver {}",d);
    }

    for d in render::drivers() {
        println!("render driver {:?}",d);
    }

    let display_mode = video_subsystem.current_display_mode(0).unwrap();
    println!("Display mode is {:?}", display_mode);

    let mut rwidth:u32 = 512;
    let mut rheight:u32 = 320;
    let mut vwidth:u32 = 512;
    let mut vheight:u32 = 320;
    if(args.width > 0) {
        vwidth = args.width;
        rwidth = args.width*args.scale;
    }
    if(args.height > 0) {
        vheight = args.height;
        rheight = args.height*args.scale;
    }
    if(args.fullscreen) {
        rwidth = display_mode.w as u32;
        rheight = display_mode.h as u32;
        vwidth = rwidth/args.scale;
        vheight = rheight/args.scale;
    }

    println!("virtual size {} {}", vwidth, vheight);
    println!("real size {} {}",rwidth,rheight);

    let mut inter1 = video_subsystem.window("rust-sdl2 demo: Video", rwidth, rheight);

    let inter2 = if args.fullscreen {
        inter1.fullscreen()
    } else {
        inter1.position_centered()
    };
    let window = inter2
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

    let canvas_builder = window.into_canvas();
    let mut canvas = canvas_builder.build().map_err(|e| e.to_string())?;
    let creator = canvas.texture_creator();

    let mut windows:HashMap<String,Window> = HashMap::new();

    let mut backend = SDL2Backend {
        sdl_context: &sdl_context,
        active_window: None,
        canvas:canvas,
        creator: &creator,
        window_buffers: Default::default(),
        window_order: vec![],
        dragging: false,
        dragtarget: None,
        resizing: false,
        font_info: load_font2("./test/font.json").unwrap(),
        scale:args.scale,
        iscale: args.scale as i32,
    };

    sdl_context.mouse().show_cursor(false);

    //channel to talk to server sender thread
    let (server_out_receive, server_out_send) = channel();
    //channel to connect server receiver thread and render loop
    let (render_loop_send, render_loop_receive) = channel::<RenderMessage>();

    let r2 = server_out_receive.clone();
    let rr2 = render_loop_send.clone();

    let receive_loop = thread::spawn(move || {
        start_connection(&args.server.clone(), r2, rr2, server_out_send, vwidth, vheight);
    });

    backend.start_loop(
        &mut windows,
        &render_loop_receive,
        &server_out_receive.clone()
    );

    println!("SDL thread is ending");
    Ok(())
}

fn start_connection(name:&str,
                    server_out_receive: Sender<OwnedMessage>,
                    render_loop_send: Sender<RenderMessage>,
                    server_out_send: Receiver<OwnedMessage>,
                    width: u32, height: u32
) {
    println!("connecting to {}",name);
    let mut client = ClientBuilder::new(name)
        .unwrap()
        .connect_insecure()
        .unwrap();

    println!("we are connected now!");
    //websocket connection
    let (mut server_in, mut server_out) = client.split().unwrap();
    //loop for receiving
    let sor = server_out_receive.clone();
    let receive_loop = thread::spawn(move || {
        process_incoming(&mut server_in, &sor, &render_loop_send);
    });

    //loop for sending
    let send_loop = thread::spawn(move || {
        process_outgoing(&server_out_send, &mut server_out);
    });

    //send the initial connection message
    let message = OwnedMessage::Text(json!(ScreenStart{
        type_: ScreenStart_name.to_string(),
    }).to_string());
    match server_out_receive.send(message) {
        Ok(()) => (),
        Err(e) => {
            println!("error sending: {:?}", e);
        }
    }

    let msg2 = OwnedMessage::Text(json!(SetScreenSize{
        type_:SetScreenSize_message.to_string(),
        width:width as i64,
        height:height as i64,
    }).to_string());
    match server_out_receive.send(msg2) {
        Ok(()) => (),
        Err(e) => {
            println!("error sending: {:?}", e);
        }
    }

    println!("Waiting for child threads to exit");
}

fn load_font<'a>(png_path: &str, json_path: &str, creator: &'a TextureCreator<WindowContext>) -> Result<FontInfo<'a>, String> {
    let font_png_1 = ImageReader::open(png_path)
        .map_err(|e|e.to_string())?
        .decode().map_err(|e|e.to_string()+"bar")?
        .into_rgba8();
    let font_texture_1 = image_to_texture_with_transparent_color(&font_png_1, &creator)?;
    let font_metrics_1 = load_json(json_path)
        .map_err(|e|e.to_string()+"baz")?;
    return Ok(FontInfo {
        bitmap: font_texture_1,
        metrics: font_metrics_1
    });
}


pub fn load_json(json_path:&str) -> Result<serde_json::Value, Box<dyn Error>> {
    let file = File::open(json_path)?;
    let reader = BufReader::new(file);
    let metrics:serde_json::Value =  serde_json::from_reader(reader)?;
    println!("metrics are object? {:?}",metrics.is_object());
    return Ok(metrics)
}
pub fn image_to_texture_with_transparent_color<'a>(rust_img:&RgbaImage, creator:&'a TextureCreator<WindowContext>) -> Result<Texture<'a>, String>{
    let mut fnt_tex2 = creator.create_texture_streaming(PixelFormatEnum::RGBA8888,
                                                        rust_img.width(),
                                                        rust_img.height())
        .map_err(|e| e.to_string())?;
    //copy the source texture, setting alpha if it's the magic bg color
    fnt_tex2.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        for y in 0..rust_img.width() {
            for x in 0..rust_img.height() {
                let ux = x as usize;
                let uy = y as usize;
                let offset = uy * pitch + ux * 4;
                let pixel = rust_img.get_pixel(x,y);
                // println!("rgb {} {} {} {}",pixel[0], pixel[1],pixel[2], pixel[3]);
                let mut alpha = 255;
                if pixel[0] == 255 && pixel[1] == 241 && pixel[2]==232 { alpha = 0; }
                buffer[offset] = alpha;
                buffer[offset + 1] = pixel[2];
                buffer[offset + 2] = pixel[1];
                buffer[offset + 3] = pixel[0];
            }
        }
    })?;

    fnt_tex2.set_blend_mode(BlendMode::Blend);
    Ok(fnt_tex2)
}
