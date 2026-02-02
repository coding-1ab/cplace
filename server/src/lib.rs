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

#[derive(serde::Deserialize, serde::Serialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy)]
#[archive_attr(derive(bytecheck::CheckBytes))]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
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
    pub pixels: [[Pixel; 64]; 64],
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            pixels: [[Pixel { r: 0, g: 0, b: 0}; 64]; 64],
        }
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
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

    chunk_data.pixels[coord.y as usize][coord.x as usize] = Pixel {
        r: pixel.r,
        g: pixel.g,
        b: pixel.b,
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
            pixel: Pixel {
                r: 255,
                g: 0,
                b: 0,
            },
            coordinate: Coordinate { x: 10, y: 10 },
        },
        PixelData {
            pixel: Pixel {
                r: 0,
                g: 255,
                b: 0,
            },
            coordinate: Coordinate { x: 32, y: 32 },
        },
        PixelData {
            pixel: Pixel {
                r: 0,
                g: 0,
                b: 255,
            },
            coordinate: Coordinate { x: 56, y: 56 },
        },
        PixelData {
            pixel: Pixel {
                r: 0,
                g: 0,
                b: 0,
            },
            coordinate: Coordinate { x: 0, y: 0 },
        },
        PixelData {
            pixel: Pixel {
                r: 0,
                g: 0,
                b: 0,
            },
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

