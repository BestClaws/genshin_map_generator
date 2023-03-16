pub struct Point {
    pub x: f32, 
    pub y: f32,
}


impl Point {
    /// creates a new point with given cartesian co-ordinates.
    pub fn new(x: f32, y: f32) -> Self {
        Self {x, y}
    }

    /// gets the absolute value of point
    /// after tranlsating the origin to the top left of the map.
    pub fn abs_point(&self, old_origin: Point) -> Self {
        Self {
            x: self.x + old_origin.x,
            y: self.y + old_origin.y,
        }
    }

}