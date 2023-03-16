mod api;
mod rect;


use image::{DynamicImage, GenericImageView};

use api::ApiClient;
use rect::Rect;


fn main2() {
    let mut map = image::open("map.jpeg").unwrap();
    let crystal =
        image::open("crystal.png")
            .unwrap()
            .resize(16, 16, image::imageops::FilterType::CatmullRom);
    let point_entries: Vec<(&DynamicImage, i64, i64)> = vec![(&crystal, 2000, 2000)];
    let result = overlay_points_hd(&map, point_entries);
    result.save("map2.jpeg").unwrap();
}

/// overlay the given image (map) with a list of images at given coords.
/// (I call these points, because that's what the official tevyat
/// interactive map calls them.)
pub fn overlay_points_hd(
    map: &DynamicImage,
    point_entries: Vec<(&DynamicImage, i64, i64)>,
) -> DynamicImage {
    // resize the map
    let (map_w, map_h) = map.dimensions();
    let mut map = map.resize(map_w * 2, map_h * 2, image::imageops::FilterType::Nearest);

    // overlay all points.
    for point_entry in point_entries {
        let (point, mut x, mut y) = point_entry;
        x *= 2;
        y *= 2;
        let point = point.resize(32, 32, image::imageops::FilterType::Nearest);
        image::imageops::overlay(&mut map, &point, x, y);
    }
    map
}



fn main() {
    let client = ApiClient::new();

    let rt = tokio::runtime::Runtime::new();

    rt.unwrap().block_on(async {
        let map_data = client.get_map_chunk(9, Rect::new(2000, 2000, 2500, 2500)).await;

        match map_data {
            Some(map_chunk) => {
                println!("saving");
                map_chunk.save("damn.jpg"); 
            }
            None => {
                println!("error: unable to create image");
            }
        }
    });
}

fn main44() {
    let x = Rect::new(4096, 4096, 8192, 8192);
        let y = Rect::new(8000, 8000, 1000, 1000);

        println!("{:#?}", x.common(&y));
}