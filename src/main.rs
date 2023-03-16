mod api;
mod point;
mod rect;


use api::client::ApiClient;
use image::{DynamicImage, GenericImageView};
use point::Point;
use rect::Rect;

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
    let client = ApiClient::new();
    client.get_map_chunk(2, Rect::new(2000, 2000, 2100, 2100));
}
