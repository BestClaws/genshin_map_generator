use crate::point::Point;

/// (lx,ly) should always be top left
/// (rx, ry) should always be bottom right
/// this is to match the data from API
/// and also easy to implement some logic.
/// TODO: add validation to match the above pattern
/// or calculate those values from given two points.
#[derive(Debug)]
pub struct Rect {
    pub lx: u32,
    pub ly: u32,
    pub rx: u32,
    pub ry: u32,
}

impl Rect {
    pub fn new(lx: u32, ly: u32, rx: u32, ry: u32) -> Self {
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

    /// returns a new rect after offseting the axes of the original rect. coords
    /// NOTE: axes should only be positive.
    /// TODO: make sure offset is only positive.
    pub fn offset_axes(&self, new_origin: Point) -> Self {
        Self {
            lx: self.lx - new_origin.x as u32,
            ly: self.ly - new_origin.y as u32,
            rx: self.rx - new_origin.x as u32,
            ry: self.ry - new_origin.y as u32,
        }
    }

    pub fn height(&self) -> u32 {
        self.ry - self.ly
    }

    pub fn width(&self) -> u32 {
        self.rx - self.lx
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test2() {
        let x = Rect::new(4096, 4096, 8192, 8192);
        let y = Rect::new(8000, 8000, 1000, 1000);

        println!("{:#?}", x.common(&y));
    }
}