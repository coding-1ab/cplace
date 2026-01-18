use mmap_sync::synchronizer::{Synchronizer, SynchronizerError};
use rocket::{get, post};
use rocket::serde::json::Json;

#[derive(serde::Deserialize)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(serde::Deserialize)]
pub struct Condinate {
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub x: i32,
    pub y: i32,
}

#[derive(serde::Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PixelData {
    pub pixel: Pixel,
    pub condinate: Condinate,
}

pub struct Chunk {
    pub pixels: [[Pixel; 64]; 64],
}

#[post("/draw",format = "json", data = "<pixel_data>")]
pub fn draw(pixel_data: Json<PixelData>) -> Result<(), SynchronizerError > {
    let mut synchronizer = Synchronizer::new("/tmp/chunk_data".as_ref());
    let data = unsafe { synchronizer.read::<Chunk>(true) }.expect("failed to read data");

    Ok(())
}

#[get("/drawing/<chunkx>/<chunky>")]
pub fn get_drawing(chunkx: i32, chunky: i32) -> Result<Json<Chunk>>, SynchronizerError> {
    
}