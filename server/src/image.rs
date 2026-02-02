use crate::PixelData;
use image::{ImageBuffer, Rgba};

const ALPHA:u8 = 0xFF;

//64 x 64 data only
pub fn generate_overlay_png(pixels: &[PixelData]) -> Result<Vec<u8>, &'static str> {
    let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(256, 256);

    for pixel in pixels {
        let x = (pixel.coordinate.x * 4) as u32;
        let y = (pixel.coordinate.y * 4) as u32;
        let r = pixel.pixel.r;
        let g = pixel.pixel.g;
        let b = pixel.pixel.b;
        for dy in 0..4 {
            for dx in 0..4 {
                img.put_pixel(x + dx, y + dy, Rgba([r, g, b, ALPHA]));
            }
        }
    }

    let mut buf = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .map_err(|_| "Could not write to PNG file")?;
    Ok(buf)
}
