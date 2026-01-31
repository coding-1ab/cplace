#![feature(trivial_bounds)]
use mmap_sync::synchronizer::{Synchronizer, SynchronizerError};
mod image;

use crate::image::generate_overlay_png;
use rocket::futures::AsyncWriteExt;
use rocket::log::private::{info, warn};
use rocket::serde::json::Json;
use rocket::{get, log, post};
use std::fs::File;
use std::io::Read;
use std::io::Write;
use rocket::http;
use rocket::http::hyper::server::conn::Http;

#[derive(serde::Deserialize, serde::Serialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub struct PixelData {
    pub pixel: Pixel,

    pub coordinate: Coordinate,
}


#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct Chunk {
    pub pixels: [[Pixel; 64]; 64],
}

#[post("/draw", format = "json", data = "<pixel_data>")]
pub fn draw(pixel_data: Json<PixelData>) -> Result<(), http::Status > {
    let mut synchronizer = Synchronizer::new("/tmp/chunk_data".as_ref());

    // rocket이 http 형태로 반환을 원해서 좀 변경 했습니다. 원본 :  let data = unsafe { synchronizer.read::<Chunk>(true) }.expect("failed to read data");
    // 이 아래부분좀 고쳐주시면 감사하겠습니다
    // let data = unsafe { synchronizer.read::<Chunk>(true) }
    //     .map_err(|_| http::Status::InternalServerError)?;

    Ok(())
}

#[get("/drawing/<chunkx>/<chunky>/<zoom_lv>")]
pub fn get_drawing(chunkx: i32, chunky: i32, zoom_lv: u8) -> Result<Vec<u8>, http::Status> {
    //위치를 기준으로 쿼리해서
    
    //sample data
    let sample_data = vec![
        PixelData {
            pixel: Pixel {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
            coordinate: Coordinate { x: 10, y: 10 },
        },
        PixelData {
            pixel: Pixel {
                r: 0,
                g: 255,
                b: 0,
                a: 255,
            },
            coordinate: Coordinate { x: 20, y: 20 },
        },
        PixelData {
            pixel: Pixel {
                r: 0,
                g: 0,
                b: 255,
                a: 255,
            },
            coordinate: Coordinate { x: 30, y: 30 },
        },
    ];

    match generate_overlay_png(&sample_data) {
        Ok(png_data) => Ok(png_data),
        Err(E) => {
            warn!("Error generating overlap png : {}",E);
            Result::Err(http::Status::InternalServerError)
        }
    }
}
