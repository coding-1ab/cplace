use crate::PixelData;
use image::{ImageBuffer, Rgba};

//64 x 64 data only
pub fn generate_overlay_png(pixels: &[PixelData]) -> Vec<u8> {
    let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(64, 64);

    for pixel in pixels {
        let x = pixel.condinate.x.round() as i32;
        let y = pixel.condinate.y.round() as i32;
        let r = pixel.pixel.r;
        let g = pixel.pixel.g;
        let b = pixel.pixel.b;
        let a = pixel.pixel.a;
        img.put_pixel(x as u32, y as u32, Rgba([r, g, b, a]));
    }

    let mut buf = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .unwrap();
    buf
}
