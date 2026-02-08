use std::io::Cursor;
use mmap_sync::synchronizer::Synchronizer;
mod image;

use crate::image::generate_overlay_png;
use rocket::{http, response, Request, Response};
use rocket::log::private::{info, warn};
use rocket::serde::json::Json;
use rocket::{get, post};
use std::time::Duration;
use rocket::http::{ContentType, Status};
use rocket::response::Responder;

pub const DEFAULT_PALETTE: [u32; 256] = [0x000000, 0x0000a8, 0x00a800, 0x00a8a8, 0xa80000, 0xa800a8, 0xa85400, 0xa8a8a8, 0x545454, 0x5454fc, 0x54fc54, 0x54fcfc, 0xfc5454, 0xfc54fc, 0xfcfc54, 0xfcfcfc, 0x000000, 0x141414, 0x202020, 0x2c2c2c, 0x383838, 0x444444, 0x505050, 0x606060, 0x707070, 0x808080, 0x909090, 0xa0a0a0, 0xb4b4b4, 0xc8c8c8, 0xe0e0e0, 0xfcfcfc, 0x0000fc, 0x4000fc, 0x7c00fc, 0xbc00fc, 0xfc00fc, 0xfc00bc, 0xfc007c, 0xfc0040, 0xfc0000, 0xfc4000, 0xfc7c00, 0xfcbc00, 0xfcfc00, 0xbcfc00, 0x7cfc00, 0x40fc00, 0x00fc00, 0x00fc40, 0x00fc7c, 0x00fcbc, 0x00fcfc, 0x00bcfc, 0x007cfc, 0x0040fc, 0x7c7cfc, 0x9c7cfc, 0xbc7cfc, 0xdc7cfc, 0xfc7cfc, 0xfc7cdc, 0xfc7cbc, 0xfc7c9c, 0xfc7c7c, 0xfc9c7c, 0xfcbc7c, 0xfcdc7c, 0xfcfc7c, 0xdcfc7c, 0xbcfc7c, 0x9cfc7c, 0x7cfc7c, 0x7cfc9c, 0x7cfcbc, 0x7cfcdc, 0x7cfcfc, 0x7cdcfc, 0x7cbcfc, 0x7c9cfc, 0xb4b4fc, 0xc4b4fc, 0xd8b4fc, 0xe8b4fc, 0xfcb4fc, 0xfcb4e8, 0xfcb4d8, 0xfcb4c4, 0xfcb4b4, 0xfcc4b4, 0xfcd8b4, 0xfce8b4, 0xfcfcb4, 0xe8fcb4, 0xd8fcb4, 0xc4fcb4, 0xb4fcb4, 0xb4fcc4, 0xb4fcd8, 0xb4fce8, 0xb4fcfc, 0xb4e8fc, 0xb4d8fc, 0xb4c4fc, 0x000070, 0x1c0070, 0x380070, 0x540070, 0x700070, 0x700054, 0x700038, 0x70001c, 0x700000, 0x701c00, 0x703800, 0x705400, 0x707000, 0x547000, 0x387000, 0x1c7000, 0x007000, 0x00701c, 0x007038, 0x007054, 0x007070, 0x005470, 0x003870, 0x001c70, 0x383870, 0x443870, 0x543870, 0x603870, 0x703870, 0x703860, 0x703854, 0x703844, 0x703838, 0x704438, 0x705438, 0x706038, 0x707038, 0x607038, 0x547038, 0x447038, 0x387038, 0x387044, 0x387054, 0x387060, 0x387070, 0x386070, 0x385470, 0x384470, 0x505070, 0x585070, 0x605070, 0x685070, 0x705070, 0x705068, 0x705060, 0x705058, 0x705050, 0x705850, 0x706050, 0x706850, 0x707050, 0x687050, 0x607050, 0x587050, 0x507050, 0x507058, 0x507060, 0x507068, 0x507070, 0x506870, 0x506070, 0x505870, 0x000040, 0x100040, 0x200040, 0x300040, 0x400040, 0x400030, 0x400020, 0x400010, 0x400000, 0x401000, 0x402000, 0x403000, 0x404000, 0x304000, 0x204000, 0x104000, 0x004000, 0x004010, 0x004020, 0x004030, 0x004040, 0x003040, 0x002040, 0x001040, 0x202040, 0x282040, 0x302040, 0x382040, 0x402040, 0x402038, 0x402030, 0x402028, 0x402020, 0x402820, 0x403020, 0x403820, 0x404020, 0x384020, 0x304020, 0x284020, 0x204020, 0x204028, 0x204030, 0x204038, 0x204040, 0x203840, 0x203040, 0x202840, 0x2c2c40, 0x302c40, 0x342c40, 0x3c2c40, 0x402c40, 0x402c3c, 0x402c34, 0x402c30, 0x402c2c, 0x40302c, 0x40342c, 0x403c2c, 0x40402c, 0x3c402c, 0x34402c, 0x30402c, 0x2c402c, 0x2c4030, 0x2c4034, 0x2c403c, 0x2c4040, 0x2c3c40, 0x2c3440, 0x2c3040, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000];

#[derive(serde::Deserialize, serde::Serialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub struct Color(u32);

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub struct PixelData {
    pub color: Color,
    pub coordinate: Coordinate,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub struct DrawRequest {
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub pixel_data: PixelData,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub struct Chunk {
    pub pixels: Vec<Vec<Color>>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            pixels: vec![vec![Color(256); 64]; 64],
        }
    }
}

fn get_chunk_path(chunk_x: i32, chunk_y: i32) -> String {
    format!("/tmp/chunk_{}_{}", chunk_x, chunk_y)
}

fn chunk_exists(chunk_x: i32, chunk_y: i32) -> bool {
    std::path::Path::new(&get_chunk_path(chunk_x, chunk_y)).exists()
}

#[post("/draw", format = "json", data = "<draw_request>")]
pub fn draw(draw_request: Json<DrawRequest>) -> http::Status {
    let chunk_x = draw_request.chunk_x;
    let chunk_y = draw_request.chunk_y;
    let coord = &draw_request.pixel_data.coordinate;
    let pixel = &draw_request.pixel_data.pixel;

    if coord.x < 0 || coord.x >= 64 || coord.y < 0 || coord.y >= 64 {
        warn!("Invalid coordinate: ({}, {})", coord.x, coord.y);
        return http::Status::BadRequest;
    }

    let chunk_path = get_chunk_path(chunk_x, chunk_y);
    let mut synchronizer = Synchronizer::new(chunk_path.as_ref());

    let mut chunk_data = if chunk_exists(chunk_x, chunk_y) {
        match unsafe { synchronizer.read::<Chunk>(true) } {
            Ok(archived) => {
                let mut pixels = [[Pixel { r: 0, g: 0, b: 0 }; 64]; 64];
                for (y, row) in archived.pixels.iter().enumerate() {
                    for (x, p) in row.iter().enumerate() {
                        pixels[y][x] = Pixel {
                            r: p.r,
                            g: p.g,
                            b: p.b,
                        };
                    }
                }
                Chunk { pixels }
            }
            Err(e) => {
                warn!("Failed to read chunk data: {:?}", e);
                return http::Status::InternalServerError;
            }
        }
    } else {
        info!("Creating new chunk at ({}, {})", chunk_x, chunk_y);
        Chunk::new()
    };

    match synchronizer.write(&chunk_data, Duration::from_secs(5)) {
        Ok(_) => {
            info!("Successfully wrote pixel at chunk ({}, {}) coord ({}, {})", 
                  chunk_x, chunk_y, coord.x, coord.y);
            http::Status::Ok
        }
        Err(e) => {
            warn!("Failed to write chunk data: {:?}", e);
            http::Status::InternalServerError
        }
    }
}

#[get("/get_pixel_png/<chunk_x>/<chunk_y>/<zoom_lv>")]
pub fn get_drawing(chunk_x: i32, chunk_y: i32, zoom_lv: u8) -> Result<PngResponse, Status> {
    let sample_data = vec![
        PixelData {
            color: Color(3),
            coordinate: Coordinate { x: 10, y: 10 },
        },
        PixelData {
            color: Color(5),
            coordinate: Coordinate { x: 32, y: 32 },
        },
        PixelData {
            color: Color(2),
            coordinate: Coordinate { x: 56, y: 56 },
        },
        PixelData {
            color: Color(1),
            coordinate: Coordinate { x: 0, y: 0 },
        },
        PixelData {
            color: Color(0),
            coordinate: Coordinate { x: 63, y: 63 },
        },
    ];

    match generate_overlay_png(&sample_data) {
        Ok(png_data) => {
            // write("test.png", &png_data).expect("TODO: panic message");
            Ok(
                PngResponse {
                    data: png_data,
                    filename: format!("{}_{}.png", chunk_x, chunk_y),
                })
        },
        Err(e) => {
            warn!("Error generating overlap png: {}", e);
            Err(Status::InternalServerError)
        }
    }
}

#[get("/get_pixel_png/<chunk_x>/<chunk_y>/<zoom_lv>")]
pub fn get_draw_test(chunk_x: i32, chunk_y: i32, zoom_lv: u8) -> Result<PngResponse, Status> {
    //todo : zoom_lv 따라서 로딩청크 동적으로 변경 및 해상도 변경 (클라이언트측과 소통 필요)
    let mut pixels_vec = Vec::with_capacity(64 * 64);

    if chunk_exists(chunk_x, chunk_y) {
        let chunk_path = get_chunk_path(chunk_x, chunk_y);
        let mut synchronizer = Synchronizer::new(chunk_path.as_ref());
        match unsafe { synchronizer.read::<Chunk>(true) } {
            Ok(archived) => {
                for (y, row) in archived.pixels.iter().enumerate() {
                    for (x, p) in row.iter().enumerate() {
                        pixels_vec.push(PixelData {
                            pixel: Pixel {
                                r: p.r,
                                g: p.g,
                                b: p.b,
                            },
                            coordinate: Coordinate { x: x as i32, y: y as i32 },
                        });
                    }
                }
            }
            Err(e) => {
                warn!("Failed to read chunk data at ({}, {}): {:?}", chunk_x, chunk_y, e);
                return Err(Status::InternalServerError);
            }
        }
    } else {
        for y in 0..64 {
            for x in 0..64 {
                pixels_vec.push(PixelData {
                    pixel: Pixel { r: 0, g: 0, b: 0 },
                    coordinate: Coordinate { x: x as i32, y: y as i32 },
                });
            }
        }
    }

    match generate_overlay_png(&pixels_vec) {
        Ok(png_data) => {
            Ok(PngResponse {
                data: png_data,
                filename: format!("{}_{}.png", chunk_x, chunk_y),
            })
        },
        Err(e) => {
            warn!("Error generating png for draw_test: {}", e);
            Err(Status::InternalServerError)
        }
    }
}


pub struct PngResponse {
    data: Vec<u8>,
    filename: String,
}

impl<'r> Responder<'r, 'static> for PngResponse {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .header(ContentType::PNG)
            .header(rocket::http::Header::new(
                "Content-Disposition",
                format!("inline; filename=\"{}\"", self.filename),
            ))
            .sized_body(self.data.len(), Cursor::new(self.data))
            .ok()
    }
}

