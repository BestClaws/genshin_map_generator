/// (lx,ly) should always be top left
/// (rx, ry) should always be bottom right
/// this is to match the data from API
/// and also easy to implement some logic.
/// TODO: add validation to match the above pattern
/// or calculate those values from given two points.
#[derive(Debug)]
pub struct Rect {
    pub lx: i32,
    pub ly: i32,
    pub rx: i32,
    pub ry: i32,
}

impl Rect {
    pub fn new(lx: i32, ly: i32, rx: i32, ry: i32) -> Self {
        Self { lx, ly, rx, ry }
    }

    pub fn common(&self, other: &Rect) -> Option<Self> {
        if other.lx < self.rx && other.ly < self.ry && other.rx > self.lx && other.ry > self.ly {
            Some(Self {
                lx: self.lx.max(other.lx),
                ly: self.ly.max(other.ly),
                rx: self.rx.min(other.rx),
                ry: self.ry.min(other.ry),
            })
        } else {
            None
        }
    }

    /// returns a new rect after translating the axes of the original rect.
    pub fn translate_axes(&self, x: i32, y: i32) -> Self {
        Self {
            lx: self.lx - x,
            ly: self.ly - y,
            rx: self.rx - x,
            ry: self.ry - y,
        }
    }

    pub fn height(&self) -> i32 {
        self.ry - self.ly
    }

    pub fn width(&self) -> i32 {
        self.rx - self.lx
    }
}

