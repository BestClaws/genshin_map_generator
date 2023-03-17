mod api;
mod shapes;
mod map_gen;

use api::client::ApiClient;
use image::{DynamicImage, GenericImageView};
use map_gen::MapGenerator;
use shapes::point::Point;
use shapes::rect::Rect;

/// overlay the given image (map) with a list of images at given coords.
/// Teyvat Interactive Map API calls these markers "Points"
pub fn overlay_markers_hd(
    map: &DynamicImage,
    marker_entries: Vec<(&DynamicImage, Point)>,
) -> DynamicImage {
    // resize the map
    let (map_w, map_h) = map.dimensions();
    let mut map = map.resize(map_w * 2, map_h * 2, image::imageops::FilterType::Nearest);

    // overlay all points.
    for marker_entry in marker_entries {
        let (marker, point) = marker_entry;
        let marker = marker.resize(32, 32, image::imageops::FilterType::Nearest);
        image::imageops::overlay(&mut map, &marker, point.x as i64, point.y as i64);
    }
    map
}

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    rt.block_on(async {
        let map_generator = MapGenerator::new();
        let image = map_generator.gen_region_map("Sea of Clouds", vec![String::from("Teleport Waypoint"), String::from("Magical Crystal Chunk")]).await.unwrap();
        image.save("done.jpg").unwrap();
    });
}