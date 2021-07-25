use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use idealos_schemas::windows::{WindowOpenDisplay, create_child_window_display, close_child_window_display};
use idealos_schemas::graphics::{DrawPixel, DrawImage, DrawRect};
use idealos_schemas::general::{Connected};




#[derive(Serialize, Deserialize, Debug)]
pub struct window_info {
    // #[serde(rename = "type")]
    pub id:String,
    pub width:i64,
    pub height:i64,
    pub x:i64,
    pub y:i64,
    pub owner:String,
    pub window_type:String,
}
pub type window_map = HashMap<String,window_info>;

#[derive(Serialize, Deserialize, Debug)]
pub struct window_list_message {
    #[serde(rename = "type")]
    pub type_:String,
    pub windows:window_map,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct set_focused_window_message {
    #[serde(rename = "type")]
    pub type_:String,
    pub window:String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct drawrect_message {
    #[serde(rename = "type")]
    pub type_:String,
    pub window:String,
    color:String,
    pub x:i64,
    pub y:i64,
    pub width:i64,
    pub height:i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct drawimage_message {
    #[serde(rename = "type")]
    pub type_:String,
    pub window:String,
    color:String,
    pub x:i64,
    pub y:i64,
    pub width:i64,
    pub height:i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum any_graphics_message {
    MAKE_DrawRect_name {
        window:String,
        color:String,
        x:i64,
        y:i64,
        width:i64,
        height:i64,
    },
    MAKE_DrawImage_name {
        window:String,
        color:String,
        x:i64,
        y:i64,
        width:i64,
        height:i64,
        depth:i64,
        channels:i64,
        pixels:Vec<u8>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct group_message {
    #[serde(rename = "type")]
    pub type_:String,
    pub category:String,
    pub messages:Vec<any_graphics_message>,
}


#[derive(Debug)]
pub enum RenderMessage {
    Connected(Connected),
    WindowList(window_list_message),
    OpenWindow(WindowOpenDisplay),
    CloseWindow(CloseWindowScreen),
    CreateChildWindow(create_child_window_display),
    CloseChildWindow(close_child_window_display),
    DrawPixel(DrawPixel),
    DrawImage(DrawImage),
    FillRect(DrawRect),
}


#[derive(Serialize, Deserialize, Debug)]
pub struct RefreshWindowMessage {
    #[serde(rename = "type")]
    pub type_:String,
    pub target:String,
    pub window:String,
}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct MouseDownMessage {
//     #[serde(rename = "type")]
//     pub type_:String,
//     pub target:String,
//     pub x:i32,
//     pub y:i32,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct MouseUpMessage {
//     #[serde(rename = "type")]
//     pub type_:String,
//     pub target:String,
//     pub x:i32,
//     pub y:i32,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct KeyboardDownMessage {
//     #[serde(rename = "type")]
//     pub type_:String,
//     pub target:String,
//     pub keyname:String,
// }


#[derive(Serialize, Deserialize, Debug)]
pub struct DrawPixelMessage {
    #[serde(rename = "type")]
    pub type_:String,
    pub color:String,
    pub window:String,
    pub x:i32,
    pub y:i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DrawImageMessage {
    #[serde(rename = "type")]
    pub type_:String,
    pub window:String,
    pub x:i32,
    pub y:i32,
    pub width:i32,
    pub height:i32,
    pub pixels: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FillRectMessage {
    #[serde(rename = "type")]
    pub type_:String,
    pub color:String,
    pub window:String,
    pub x:i32,
    pub y:i32,
    pub width:i32,
    pub height:i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenWindowScreen {
    #[serde(rename = "type")]
    pub type_:String,
    pub target:String,
    pub window: WindowInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CloseWindowScreen {
    #[serde(rename = "type")]
    pub type_:String,
    pub target:String,
    pub window: WindowInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WindowInfo {
    pub id:String,
    pub x:i32,
    pub y:i32,
    pub width:i32,
    pub height:i32,
    pub owner:String,
    pub window_type:String,
}





pub const MouseDown_name: &str = "MAKE_MouseDown_name";
#[derive(Serialize, Deserialize, Debug)]
pub struct MouseDown {
    #[serde(rename = "type")]
    pub type_:String,
    pub x:i64,
    pub y:i64,
    pub target:String,
    pub window:String,
}

pub const MouseUp_name: &str = "MAKE_MouseUp_name";
#[derive(Serialize, Deserialize, Debug)]
pub struct MouseUp {
    #[serde(rename = "type")]
    pub type_:String,
    pub x:i64,
    pub y:i64,
    pub target:String,
    pub window:String,
}


pub const KeyboardDown_name: &str = "MAKE_KeyboardDown_name";
#[derive(Serialize, Deserialize, Debug)]
pub struct KeyboardDown {
    #[serde(rename = "type")]
    pub type_:String,
    pub code:String,
    pub target:String,
    pub app:String,
    pub window:String,
    pub key:String,
    pub shift:bool,
    pub alt:bool,
    pub meta:bool,
    pub control:bool,
}



pub const WindowSetPosition_message: &str = "MAKE_WindowSetPosition_name";
#[derive(Serialize, Deserialize, Debug)]
pub struct WindowSetPosition {
    #[serde(rename = "type")]
    pub type_:String,
    pub app:String,
    pub window:String,
    pub x:i64,
    pub y:i64,
}


pub const WindowSetSize_message: &str = "window-set-size";
#[derive(Serialize, Deserialize, Debug)]
pub struct WindowSetSize {
    #[serde(rename = "type")]
    pub type_:String,
    pub app:String,
    pub window:String,
    pub width:i64,
    pub height:i64,
}

