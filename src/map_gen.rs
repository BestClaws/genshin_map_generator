use image::{DynamicImage, GenericImageView, RgbaImage};
use serde_json::map::Iter;

use crate::{
    api::{
        client::ApiClient,
        models::{Label, Marker, MarkerData, RegionData},
    },
    shapes::{point::Point, rect::Rect},
};

/// overlay the given image (map) with a list of images at given coords.
/// Teyvat Interactive Map API calls these markers "Points"
pub fn overlay_markers_hd(
    map: &mut DynamicImage,
    images_with_marker_points: Vec<(DynamicImage, impl Iterator<Item = Point>)>,
) {
    let marker_bg = image::open("marker_bg.png").unwrap();

    for image_with_marker_points in images_with_marker_points {
        let (image, marker_points) = image_with_marker_points;
        for marker_point in marker_points {
            let image = image.resize(32, 32, image::imageops::FilterType::CatmullRom);
            image::imageops::overlay(
                map,
                &marker_bg,
                marker_point.x as i64 - 16,
                marker_point.y as i64 - 32,
            );
            image::imageops::overlay(
                map,
                &image,
                marker_point.x as i64 - 16,
                marker_point.y as i64 - 32,
            );
        }
    }
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

    /// generates the map for a given region (not a sub region)
    pub async fn gen_region_map(
        &self,
        region_name: &str,
        desired_marker_labels: Vec<String>,
    ) -> anyhow::Result<Option<DynamicImage>> {
        let map_ids: Vec<u8> = self.client.fetch_map_ids().await?;

        let mut region_result: Option<(RegionData, u8)> = None;

        

        'map_search: for map_id in map_ids {
        
            let regions = self.client.fetch_regions(map_id).await?;
            for region in regions {
                if region.name.contains(region_name) {
                    // region matched
                    region_result = Some((region, map_id));
                    break 'map_search;
                }

                for sub_region in region.children {
                    if !sub_region.name.contains(region_name) {
                        // sub region dint match try next subregion.
                        continue;
                    }
                    println!("yes");
                    // sub region matched, get out and start processing.
                    region_result = Some((sub_region, map_id));
                    break 'map_search;
                }
            }
        }

        let Some((region, map_id)) = region_result else {
            return Ok(None);
        };

        let map_data = self.client.fetch_map_data(map_id).await?;
        let marker_data = self.client.fetch_marker_data(map_id).await.unwrap();
        let frame = region.get_abs_frame(map_data.origin());
        let mut map_chunk = self.client.get_map_chunk(&map_data, &frame).await.unwrap();

        let matched_labels: Vec<&Label> = marker_data
            .labels
            .iter()
            .filter(|label| {
                desired_marker_labels
                    .iter()
                    .any(|desired_label| label.name.contains(desired_label))
            })
            .collect();

        println!("a");

        let mut matched_markers = vec![];

        for label in matched_labels {
            let image = self.client.fetch_image(&label.icon).await.unwrap();

            let matched_marker_points = marker_data
                .markers
                .iter()
                .filter(|marker| marker.label_id == label.id)
                .map(|marker| {
                    marker
                        .pos()
                        .abs_point(map_data.origin())
                        .translate_axes(frame.top_left())
                });

            matched_markers.push((image, matched_marker_points));
        }

        overlay_markers_hd(&mut map_chunk, matched_markers);

        return Ok(Some(map_chunk));
    }

    pub async fn gen_area_map(
        &self,
        region_name: &str,
        desired_marker_labels: Vec<String>,
    ) -> anyhow::Result<Option<DynamicImage>> {

        let areas = self.client.fetch_areas().await?;

       
        let mut matched_area_regions = Vec::new();

        let mut map_id = 0;


        for (area_id, area) in (1..).zip(areas.iter()) {
            if area.name.contains(region_name) {
                println!("sumeru: {}", area_id);
                let regions = self.client.fetch_regions(area.map_id).await?;

                let map_data = self.client.fetch_map_data(area.map_id).await?;
               

                for region in regions {

                    if region.area_id == area_id as u8 {

                        let frame = region.get_abs_frame(map_data.origin());
                        println!("my frame: {frame:?}");
                        matched_area_regions.push(region);
                    }
                }

                if !matched_area_regions.is_empty() {
                    map_id = area.map_id;
                    break;
                }
            }
        }

        let map_data = self.client.fetch_map_data(map_id).await?;

        // calculate the frame
        let lx = matched_area_regions.iter().fold(1000000, |lx, region| {
            if region.get_abs_frame(map_data.origin()).lx < lx {
                region.get_abs_frame(map_data.origin()).lx
            } else { lx }
        });

        let ly = matched_area_regions.iter().fold(1000000, |ly, region| {
            if region.get_abs_frame(map_data.origin()).ly < ly {
                region.get_abs_frame(map_data.origin()).ly
            } else { ly }
        });

        let rx = matched_area_regions.iter().fold(0, |rx, region| {
            if region.get_abs_frame(map_data.origin()).rx > rx {
                region.get_abs_frame(map_data.origin()).rx
            } else { rx }
        });

        let ry = matched_area_regions.iter().fold(0, |ry, region| {
            if region.get_abs_frame(map_data.origin()).ry > ry {
                region.get_abs_frame(map_data.origin()).ry
            } else { ry }
        });



        let frame = Rect::new(lx as u32, ly  as u32, rx  as u32, ry  as u32);

        println!("frame: {:?}", frame);

        println!("mapid: {}", map_id);
        
  
  
        let map_data = self.client.fetch_map_data(map_id).await?;
        let marker_data = self.client.fetch_marker_data(map_id).await.unwrap();
        let mut map_chunk = self.client.get_map_chunk(&map_data, &frame).await.unwrap();

        let matched_labels: Vec<&Label> = marker_data
            .labels
            .iter()
            .filter(|label| {
                desired_marker_labels
                    .iter()
                    .any(|desired_label| label.name.contains(desired_label))
            })
            .collect();

        println!("a");

        let mut matched_markers = vec![];

        for label in matched_labels {
            let image = self.client.fetch_image(&label.icon).await.unwrap();

            let matched_marker_points = marker_data
                .markers
                .iter()
                .filter(|marker| marker.label_id == label.id)
                .map(|marker| {
                    marker
                        .pos()
                        .abs_point(map_data.origin())
                        .translate_axes(frame.top_left())
                });

            matched_markers.push((image, matched_marker_points));
        }

        overlay_markers_hd(&mut map_chunk, matched_markers);

        Ok(Some(map_chunk))
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

            match image {
                Some(image) => {
                    image.save("done.jpg").unwrap();
                }
                None => {
                    println!("no matches");
                }
            }
        });
    }
}
