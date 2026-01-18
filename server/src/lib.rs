use mmap_sync::synchronizer::{Synchronizer, SynchronizerError};
mod image;

use crate::image::generate_overlay_png;
use memmap::Mmap;
use rocket::futures::AsyncWriteExt;
use rocket::log::private::info;
use rocket::serde::json::Json;
use rocket::{get, post};
use std::fs::File;
use std::io::Read;
use std::io::Write;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Coordinate {
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub x: i32,
    pub y: i32,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub struct PixelData {
    pub pixel: Pixel,
    pub coordinate: Coordinate,
}

pub struct Chunk {
    pub pixels: [[Pixel; 64]; 64],
}

pub struct Chunk {
    pub pixels: [[Pixel; 64]; 64],
}

#[post("/draw", format = "json", data = "<pixel_data>")]
pub fn draw(pixel_data: Json<PixelData>) -> Result<(), SynchronizerError > {
    let mut synchronizer = Synchronizer::new("/tmp/chunk_data".as_ref());
    let data = unsafe { synchronizer.read::<Chunk>(true) }.expect("failed to read data");

    Ok(())
}

#[get("/drawing/<chunkx>/<chunky>/<zoom_lv>")]
pub fn get_drawing(chunkx: i32, chunky: i32, zoom_lv: u8) -> Result<Json<Vec<PixelData>>, ()> {
    //

    //example
    Ok(Json(vec![PixelData {
        pixel: Pixel {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        },
        coordinate: Coordinate {x: 0.0, y: 0.0},
    }]))
}



//test and usage guide function
#[get("/px_data_to_png_test")]
pub fn px_data_to_png_test() {
    info!("px_data_to_png_test");
    let sample_data = vec![
        PixelData {
            pixel: Pixel {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
            coordinate: Coordinate { x: 10.0, y: 10.0 },
        },
        PixelData {
            pixel: Pixel {
                r: 0,
                g: 255,
                b: 0,
                a: 255,
            },
            coordinate: Coordinate { x: 20.0, y: 20.0 },
        },
        PixelData {
            pixel: Pixel {
                r: 0,
                g: 0,
                b: 255,
                a: 255,
            },
            coordinate: Coordinate { x: 30.0, y: 30.0 },
        },
    ];

    let png_data: Vec<u8> = generate_overlay_png(&sample_data);
    let mut file = File::create("test.png".to_string()).unwrap();
    let _ = file.write_all(&png_data);
}
