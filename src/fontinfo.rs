use sdl2::render::{Texture, WindowCanvas};
use sdl2::pixels::Color;
use sdl2::rect::Rect;

pub struct FontInfo<'a> {
    pub bitmap: Texture<'a>,
    pub metrics: serde_json::Value
}

impl FontInfo<'_> {
    pub fn ascent(&self) -> i64 {
        return self.metrics.as_object().unwrap().get("ascent").unwrap().as_i64().unwrap();
    }

    pub fn draw_text_at(&self, text: &str, x:i32, y:i32, color:&Color, canvas: &mut WindowCanvas, scale_i:i32) {
        let mut dx:i32 = x;
        let mut dy:i32 = y;
        let scale_u:u32 = scale_i as u32;

        for ch in text.chars() {
            let arr = self.metrics.as_object().unwrap().get("metrics").unwrap().as_array().unwrap();
            if (ch as usize) < arr.len() {
                let met_res = arr[ch as usize].as_object();
                if let Some(met) = met_res {

                    // println!("char is {} {:?}",ch,met);
                    let sx: i32 = met.get("x").unwrap().as_u64().unwrap() as i32;
                    let sy: i32 = met.get("y").unwrap().as_u64().unwrap() as i32;
                    let sw: u32 = met.get("w").unwrap().as_u64().unwrap() as u32;
                    let sh: u32 = met.get("h").unwrap().as_u64().unwrap() as u32;
                    let bl: u32 = met.get("baseline").unwrap().as_u64().unwrap() as u32;

                    let src = Rect::new(sx, sy, sw, sh);
                    let dst = Rect::new(
                        dx * scale_i,
                        (y + (bl as i32)) * scale_i as i32,
                        sw * scale_u,
                        sh * scale_u);
                    canvas.copy(&self.bitmap, src, dst);
                    dx += sw as i32;
                    dx += 1;
                    continue;
                }
            }
            let missing = Rect::new(dx*scale_i,dy*scale_i,10*scale_u,10*scale_u);
            canvas.set_draw_color(Color::BLACK);
            canvas.draw_rect(missing);
            dx += 11;
            continue;
        }
    }
}