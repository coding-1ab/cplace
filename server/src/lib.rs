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
    pub x: f32,
    pub y: f32,
}

#[derive(serde::Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PixelData {
    pub pixel: Pixel,
    pub condinate: Condinate,
}

#[post("/draw",format = "json", data = "<pixel_data>")]
pub fn draw(pixel_data: Json<PixelData>) -> Result<(), ()> {
    todo!();
}

#[get("/drawing/<chunkx>/<chunky>")]
pub fn get_drawing(chunkx: i32, chunky: i32) -> Result<Json<Vec<PixelData>>, ()> {
    todo!();
}