use rocket::{get, post};
use rocket::serde::json::Json;

// R3G3B2
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Pixel(pub u8);

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Coordinate {
    pub x: f32,
    pub y: f32,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub struct PixelData {
    pub pixel: Pixel,
    pub coordinate: Coordinate,
}

#[post("/draw",format = "json", data = "<pixel_data>")]
pub fn draw(pixel_data: Json<PixelData>) -> Result<(), ()> {
    todo!();
}

#[get("/drawing/<chunkx>/<chunky>")]
pub fn get_drawing(chunkx: i32, chunky: i32) -> Result<Json<Vec<PixelData>>, ()> {
    todo!();
}