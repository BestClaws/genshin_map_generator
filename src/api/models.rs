use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::shapes::{point::Point, rect::Rect};

#[derive(Debug, Serialize, Deserialize)]
pub struct MapData {
    pub slices: Vec<Vec<HashMap<String, String>>>,
    origin: (f32, f32),
    total_size: (i32, i32),
    pub padding: (f32, f32),
}

impl MapData {
    pub fn origin(&self) -> Point {
        Point {
            x: self.origin.0,
            y: self.origin.1,
        }
    }

    pub fn padding(&self) -> Point {
        Point {
            x: self.padding.0,
            y: self.padding.1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AreaData {
    name: String,
    #[serde(rename = "pc_icon_url")]
    pub icon_url: String,
    #[serde(rename = "l_x")]
    lx: f32,
    #[serde(rename = "l_y")]
    ly: f32,
    #[serde(rename = "r_x")]
    rx: f32,
    #[serde(rename = "r_y")]
    ry: f32,
    pub map_id: u8,
}

impl AreaData {
    /// converts the relative frame from origin to absolute (from top left of the map)
    fn get_abs_frame(&self, old_origin: Point) -> Rect {
        // origin = (h,k)
        // new_origin = (-h,-k); basically shifting back the origin to 0, 0 (top left of the map)
        // (X,Y) = (x - (-h), y - (-k)); new coorinates with respect to new origin
        // (X,Y) = (x + h, y + k)
        Rect {
            // WARNING. chances of overflows of converting i32 to u32
            // I'm just trusting the API here. PLEASE add a verification
            // step here and panic for such cases.
            lx: (self.lx + old_origin.x) as u32,
            ly: (self.ly + old_origin.y) as u32,
            rx: (self.rx + old_origin.x) as u32,
            ry: (self.ry + old_origin.y) as u32,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegionData {
    pub name: String,
    #[serde(rename = "l_x")]
    lx: f32,
    #[serde(rename = "l_y")]
    ly: f32,
    #[serde(rename = "r_x")]
    rx: f32,
    #[serde(rename = "r_y")]
    ry: f32,
    pub area_id: u8,
    pub children: Vec<RegionData>,
}

// TODO: duplication? excuse me?
impl RegionData {
    /// returns the frame after translating origin to top left of the map.
    pub fn get_abs_frame(&self, old_origin: Point) -> Rect {
        // origin = (h,k)
        // new_origin = (-h,-k); basically shifting back the origin to 0, 0 (top left of the map)
        // (X,Y) = (x - (-h), y - (-k)); new coorinates with respect to new origin
        // (X,Y) = (x + h, y + k)
        Rect {
            // WARNING. chances of overflows of converting i32 to u32
            // I'm just trusting the API here. PLEASE add a verification
            // step here and panic for such cases.
            lx: (self.lx + old_origin.x) as u32,
            ly: (self.ly + old_origin.y) as u32,
            rx: (self.rx + old_origin.x) as u32,
            ry: (self.ry + old_origin.y) as u32,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MarkerData {
    #[serde(rename = "point_list")]
    pub markers: Vec<Marker>,
    #[serde(rename = "label_list")]
    pub labels: Vec<Label>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Marker {
    pub label_id: i32,
    pub area_id: u8,
    #[serde(rename = "x_pos")]
    x: f32,
    #[serde(rename = "y_pos")]
    y: f32,
}

impl Marker {
    pub fn pos(&self) -> Point {
        Point::new(self.x, self.y)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Label {
    pub name: String,
    pub icon: String,
    pub id: i32,
}
