use serde::{Deserialize, Serialize};
use sdl2::video::WindowContext;
use sdl2::render::TextureCreator;
use std::fs::{File, read_to_string};
use std::io::BufReader;
use std::error::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GlyphInfo {
    pub id:u32,
    pub name:String,
    pub width:i32,
    pub height:i32,
    pub baseline:i32,
    pub data:Vec<u32>,
    pub ascent:i32,
    pub descent:i32,
    pub left:i32,
    pub right:i32,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FontInfo2 {
    pub name:String,
    pub glyphs:Vec<GlyphInfo>,
}

// Result<serde_json::Value, Box<dyn Error>>
pub fn load_font2<'a>(json_path: &str) -> Result<FontInfo2, Box<dyn Error>> {
    let txt = read_to_string(json_path)?;
    let font:FontInfo2 = serde_json::from_str(txt.as_str())?;
    return Ok(font)
}
