use image::{DynamicImage, GenericImageView, RgbaImage};

use crate::{
    api::{
        client::ApiClient,
        models::{Label, Marker, MarkerData, RegionData},
    },
    shapes::point::Point,
};

/// overlay the given image (map) with a list of images at given coords.
/// Teyvat Interactive Map API calls these markers "Points"
pub fn overlay_markers_hd(
    map: DynamicImage,
    marker_entries: Vec<(DynamicImage, Point)>,
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

pub struct MapGenerator {
    client: ApiClient,
    // maker_data: MarkerData,
}

impl MapGenerator {
    pub fn new() -> Self {
        let client = ApiClient::new();
        Self { client }
    }

    pub async fn gen_region_map(
        &self,
        region_name: &str,
        desired_marker_labels: Vec<String>,
    ) -> Option<DynamicImage> {
        let map_ids: Vec<u8> = match self.client.fetch_map_ids().await {
            Ok(map_ids) => map_ids,
            Err(e) => {
                println!("error trying to fetch region: {e}");
                panic!("fix this later. you should panic by report error (required when python bindings are implemented.")
            }
        };

        for map_id in map_ids {
            println!("map: {}", map_id);
            let regions = match self.client.fetch_regions(map_id).await {
                Ok(regions) => regions,
                Err(e) => {
                    println!("error trying to fetch regoins: {e}");
                    panic!("you should be reporting this error instead.");
                }
            };

            println!("using map id: {}", map_id);
            let map_data = match self.client.fetch_map_data(map_id).await {
                Ok(map_data) => map_data,
                Err(e) => {
                    println!("unable to fetch map data: {e}");
                    panic!("you should be reportng this error instead");
                }
            };

            for region in regions {
                println!(
                    "processing region: {}, desired: {}",
                    region.name, region_name
                );
                if !region.name.contains(region_name) {
                    continue;
                }

                let frame = region.get_abs_frame(map_data.origin());
                let map_chunk = self.client.get_map_chunk(&map_data, &frame).await.unwrap();

                let marker_data = self.client.fetch_marker_data(map_id).await.unwrap();

                let mut matched_labels: Vec<Label> = vec![];

                for desired_marker_label in desired_marker_labels {
                    for label in &marker_data.labels {
                        if label.name.contains(&desired_marker_label) {
                            matched_labels.push(label.clone());
                        }
                    }
                }

                let mut matched_markers: Vec<(DynamicImage, Point)> = vec![];

                for marker in marker_data.markers {
                    for label in &matched_labels {
                        if marker.label_id == label.id {
                            let image = self.client.fetch_image(&label.icon).await.unwrap();

                            let point = marker.pos();
                            println!("marker pos: {:?}", point);
                            let orig = map_data.origin();
                            println!("orig: {:?}", orig);
                            let point = point.abs_point(orig);
                            println!(" new point: {:?}", point);
                            let frame_top_left = frame.top_left();
                            println!("frame top left: {:?}", frame_top_left);
                            let point = point.translate_axes(frame.top_left());
                            println!("finally point: {:?}", point);

                            matched_markers.push((
                                image,
                                // marker.pos().abs_point(map_data.origin()).translate_axes(frame.top_left()).translate_axes(map_data.padding())
                                point,
                            ))
                        }
                    }
                }

                println!("matched labels: {:#?}", matched_labels);

                println!("matched markers: {:#?}", matched_markers.len());

                let res = overlay_markers_hd(map_chunk, matched_markers);

                return Some(res);
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use super::MapGenerator;

    #[test]
    fn test_gen_region_map() {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let map_generator = MapGenerator::new();
            let image = map_generator
                .gen_region_map("Enkanomiya", Vec::new())
                .await
                .unwrap();
            image.save("done.jpg").unwrap();
        });
    }
}
