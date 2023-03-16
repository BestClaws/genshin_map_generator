use std::collections::HashMap;

use image::DynamicImage;
use image::GenericImageView;
use image::ImageBuffer;
use image::RgbaImage;

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use serde::{Deserialize, Serialize};

use super::rect::Rect;

#[derive(Debug, Serialize, Deserialize)]
pub struct MapData {
    slices: Vec<Vec<HashMap<String, String>>>,
    origin: (i32, i32),
    total_size: (i32, i32),
    padding: (i32, i32),
}

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

    /// fetches the MapData for given map_id
    async fn fetch_map_data(&self, map_id: i32) -> anyhow::Result<MapData> {
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
    async fn fetch_image(&self, url: &str) -> anyhow::Result<DynamicImage> {
        let bytes = self.client.get(url).send().await?.bytes().await?;

        let reader = image::io::Reader::new(std::io::Cursor::new(bytes))
            .with_guessed_format()
            .expect("cursor io never fails");

        Ok(reader.decode()?)
    }

    pub async fn get_map_chunk(&self, map_id: i32, frame: Rect) -> Option<RgbaImage> {
        // TODO: this is unncessary allocation if frame doesn't fit anywhere in map
        let mut output: RgbaImage =
            ImageBuffer::new((frame.rx - frame.lx) as u32, (frame.ry - frame.ly) as u32);

        let Ok(map_data) = self.fetch_map_data(map_id).await else {
            println!("error: could not fetch map data");
            return None;
        };

        for (y, row) in map_data.slices.iter().enumerate() {
            for (x, map_chunk) in row.iter().enumerate() {
                let x = x as u32;
                let y = y as u32;

                let url = map_chunk.get("url").unwrap();

                let Ok(map_chunk) =
                    self.fetch_image(url).await else {
                        println!("error: could not fetch image");
                        return None;
                    };
                let (width, height) = map_chunk.dimensions();

                let map_chunk_r = Rect::new(
                    (x * width) as i32,
                    (y * height) as i32,
                    ((x + 1) * width) as i32,
                    ((y + 1) * height) as i32,
                );

                let extracted_chunk_r = map_chunk_r.common(&frame);
                println!("common: {:?}", extracted_chunk_r);

                let Some(extracted_chunk_r) = extracted_chunk_r else {
                    continue;
                };

                // offset from map chunk
                let extracted_chunk_mc_r =
                    extracted_chunk_r.translate_axes(map_chunk_r.lx, map_chunk_r.ly);

                // offset from frame.
                let output_chunk_f_r = extracted_chunk_r.translate_axes(frame.lx, frame.ly);

                // TODO: .to_image() seems expensive.
                let extracted_chunk = map_chunk.view(
                    extracted_chunk_mc_r.lx as u32,
                    extracted_chunk_mc_r.ly as u32,
                    extracted_chunk_mc_r.width() as u32,
                    extracted_chunk_mc_r.height() as u32,
                ).to_image();

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
        Some(output)
    }
}

#[cfg(test)]
mod test {
    use super::ApiClient;
    use super::*;

    #[test]
    fn test() {
        let client = ApiClient::new();

        let rt = tokio::runtime::Runtime::new();

        rt.unwrap().block_on(async {
            let map_data = client
                .get_map_chunk(2, Rect::new(8000, 8000, 100, 100))
                .await;

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

    #[test]
    fn test2() {
        let x = Rect::new(4096, 4096, 8192, 8192);
        let y = Rect::new(8000, 8000, 1000, 1000);

        println!("{:#?}", x.common(&y));
    }
}
