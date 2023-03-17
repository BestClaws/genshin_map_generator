use std::collections::HashMap;

use image::DynamicImage;
use image::GenericImageView;
use image::ImageBuffer;
use image::RgbaImage;

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use serde::{Deserialize, Serialize};

use crate::shapes::point::Point;
use crate::shapes::rect::Rect;

use super::models::MapData;
use super::models::AreaData;
use super::models::MarkerData;
use super::models::RegionData;

pub struct ApiClient {
    client: ClientWithMiddleware,
}

impl ApiClient {
    // returns a new instance of client.
    // internally this is a new http reqwest client with caching middleware.
    pub fn new() -> Self {
        let client = ClientBuilder::new(Client::new())
            .with(Cache(HttpCache {
                mode: CacheMode::Default,
                manager: CACacheManager::default(),
                options: None,
            }))
            .build();

        Self { client }
    }

    pub async fn fetch_map_ids(&self) -> anyhow::Result<Vec<u8>>{
        let url = "https://sg-public-api.hoyolab.com/common/map_user/ys_obc/v1/map/list?app_sn=ys_obc&lang=en-us";
        let response: serde_json::Value = self.client.get(url).send().await?.json().await?;
        // TODO: this breaks if api changes (shouldn't happen since versioning is used.)
        let data = &response["data"]["list"].as_array().unwrap();
        let data: Vec<u8> = data.iter().map(|val| val["id"].as_u64().unwrap() as u8).collect();

    
        Ok(data)
    }

    /// fetches the MapData for given map_id
    pub async fn fetch_map_data(&self, map_id: u8) -> anyhow::Result<MapData> {
        let url = format!("https://sg-public-api-static.hoyolab.com/common/map_user/ys_obc/v1/map/info?map_id={map_id}&app_sn=ys_obc&lang=en-us");
        let response: serde_json::Value = self.client.get(url).send().await?.json().await?;
        // TODO: this breaks if api changes (shouldn't happen since versioning is used.)
        let data = &response["data"]["info"]["detail"].as_str().unwrap();
        // TODO: this might break if the structure changes (shouldn't happen snice
        // versoning is used.)
        let map_data: MapData = serde_json::from_str(&data).unwrap();
        Ok(map_data)
    }

    /// fetches the image (map) for the given URL
    pub async fn fetch_image(&self, url: &str) -> anyhow::Result<DynamicImage> {
        let bytes = self.client.get(url).send().await?.bytes().await?;
     
        let reader = image::io::Reader::new(std::io::Cursor::new(bytes))
            .with_guessed_format()
            .expect("cursor io never fails");

        let image = reader.decode()?;// RIDICULOUSLY SLOW. WHAT IS IT EVEN DOING
        Ok(image) 
    }

    // TODO: this better be result as network is involved.
    pub async fn get_map_chunk(&self, map_data: &MapData, frame: &Rect) -> Option<DynamicImage> {
        // TODO: this is unncessary allocation if frame doesn't fit anywhere in map
        
        let mut output: RgbaImage =
            ImageBuffer::new((frame.rx - frame.lx) as u32, (frame.ry - frame.ly) as u32);

        let mut map_chunk_dimensions: Option<(u32, u32)> = None;

        // FROM HERE ONWARDS _r means the rect variant
        for (y, row) in (0..).zip(map_data.slices.iter()) {
            for (x, map_chunk) in (0..).zip(row.iter()) {
                let url = map_chunk.get("url").unwrap();

                // use already present dimensoins or fetch image and calculate dimension.
                // double fetch for y=0, x =0 iteration here.
                // atleast better than old version which hit fetch_image on every iteration.
                let (width, height)  = match map_chunk_dimensions  {
                    Some((width, height))=> {
                        (width, height)
                    }
                    None => {
                        let Ok(map_chunk) =
                            self.fetch_image(url).await else {
                                println!("error: could not fetch image");
                                return None;
                            };
                        map_chunk_dimensions = Some(map_chunk.dimensions());
                        map_chunk.dimensions()
                        
                    }

                };

                let map_chunk_r =
                    Rect::new(x * width, y * height, (x + 1) * width, (y + 1) * height);
                

                println!("map chunk: {:?}", map_chunk_r);
                println!("frame: {:?}", frame);

                // the common rect between map chunk and given frame
                let Some(extracted_chunk_r) = map_chunk_r.common(&frame) else {
                    continue;
                };

                let Ok(map_chunk) =
                self.fetch_image(url).await else {
                    println!("error: could not fetch image");
                    return None;
                };
                
                println!("common: {:?}", extracted_chunk_r);

                // offset from map chunk
                let extracted_chunk_mc_r = extracted_chunk_r
                    .translate_axes(Point::new(map_chunk_r.lx as f32, map_chunk_r.ly as f32));

                // offset from frame.
                let output_chunk_f_r =
                    extracted_chunk_r.translate_axes(Point::new(frame.lx as f32, frame.ly as f32));

                // TODO: .to_image() seems expensive.
                let extracted_chunk = map_chunk
                    .view(
                        extracted_chunk_mc_r.lx,
                        extracted_chunk_mc_r.ly,
                        extracted_chunk_mc_r.width(),
                        extracted_chunk_mc_r.height(),
                    )
                    .to_image();

                image::imageops::replace(
                    &mut output,
                    &extracted_chunk,
                    output_chunk_f_r.lx.into(),
                    output_chunk_f_r.ly.into(),
                );
            }
        }

   

        // TODO: IF NO matches found. then white image
        // if if frame is partially outside of map. then partial image
        // will be rendered.
        Some(DynamicImage::ImageRgba8(output))
    }

    pub async fn fetch_areas(&self) -> anyhow::Result<Vec<AreaData>> {
        let url = "https://sg-public-api-static.hoyolab.com/common/map_user/ys_obc/v1/map/get_area_pageLabel?map_id=9&app_sn=ys_obc&lang=en-us";
        let mut response: serde_json::Value = self.client.get(url).send().await?.json().await?;

        let areas: Vec<AreaData> = serde_json::from_value(response["data"]["list"].take())?;

        Ok(areas)
    }

    pub async fn fetch_regions(&self, map_id: u8) -> anyhow::Result<Vec<RegionData>> {
        let url = format!("https://sg-public-api-static.hoyolab.com/common/map_user/ys_obc/v1/map/map_anchor/list?map_id={map_id}&app_sn=ys_obc&lang=en-us");

        let mut response: serde_json::Value = self.client.get(url).send().await?.json().await?;

        let regions: Vec<RegionData> = serde_json::from_value(response["data"]["list"].take())?;

        Ok(regions)
    }

    // very expensive deserialization. cache it in Apiclient (not the data but the deserialized data.)
    pub async fn fetch_marker_data(&self, map_id: u8) -> anyhow::Result<MarkerData> {
        let url = format!("https://sg-public-api-static.hoyolab.com/common/map_user/ys_obc/v1/map/point/list?map_id={map_id}&app_sn=ys_obc&lang=en-us");

        let mut response:  serde_json::Value = self.client.get(url).send().await?.json().await?;
        let  marker_data: MarkerData = serde_json::from_value(response["data"].take())?;
        Ok(marker_data)
    }
}

#[cfg(test)]
mod test {
    use super::ApiClient;
    use super::*;

    
    #[test]
    fn test_fetch_map_ids() {
        let client = ApiClient::new();
        let rt = tokio::runtime::Runtime::new();

        rt.unwrap().block_on(async {
            match client.fetch_map_ids().await {
                Ok(map_ids) => {
                    println!("map ids: {:#?}", map_ids);
                }
                Err(e) => {
                    println!("error occured: {:?}", e);
                }
            }
        });
    }




    #[test]
    fn test_get_map_chunk() {
        let client = ApiClient::new();

        let rt = tokio::runtime::Runtime::new();

        rt.unwrap().block_on(async {
            let map_data = client.fetch_map_data(2).await.unwrap();
            let map_chunk = client
                .get_map_chunk(&map_data, &Rect::new(8000, 8000, 8100, 8100))
                .await;

            match map_chunk {
                Some(i) => {
                    i.save("test2.png").unwrap();
                    println!("image was generated succesfully");
                }
                None => {
                    println!("error: unable to create image");
                }
            }
        });
    }

    #[test]
    fn test_fetch_areas() {
        let client = ApiClient::new();
        let rt = tokio::runtime::Runtime::new();

        rt.unwrap().block_on(async {
            match client.fetch_areas().await {
                Ok(areas) => {
                    println!("{:#?}", areas);
                }
                Err(e) => {
                    println!("error occured: {:?}", e);
                }
            }
        });
    }

    #[test]
    fn test_fetch_regions() {
        let client = ApiClient::new();
        let rt = tokio::runtime::Runtime::new();

        rt.unwrap().block_on(async {
            match client.fetch_regions(2).await {
                Ok(areas) => {
                    println!("{:#?}", areas);
                }
                Err(e) => {
                    println!("error occured: {:?}", e);
                }
            }
        });
    }

    #[test]
    fn test_fetch_marker_data() {
        let client = ApiClient::new();
        let rt = tokio::runtime::Runtime::new();

        rt.unwrap().block_on(async {
            match client.fetch_marker_data(2).await {
                Ok(marker_data) => {
                    println!("success");
                }
                Err(e) => {
                    println!("error occured: {:?}", e);
                }
            }
        });
    }
}
