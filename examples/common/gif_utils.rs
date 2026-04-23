use gif::{Encoder, Frame, Repeat};
use image::ImageReader;
use std::fs::File;
use std::io::BufWriter;

use super::rendering::get_color_palette;

pub const FRAME_DELAY: u16 = 4;

pub fn setup_gif(
    path: &str,
    width: u16,
    height: u16,
) -> Result<Encoder<BufWriter<File>>, Box<dyn std::error::Error>> {
    let palette = get_color_palette();
    let flat_palette: Vec<u8> = palette.iter().flat_map(|c| c.iter().copied()).collect();

    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let mut encoder = Encoder::new(writer, width, height, &flat_palette)?;
    encoder.set_repeat(Repeat::Infinite)?;

    Ok(encoder)
}

pub fn find_closest_color(pixel: &[u8; 3], palette: &[[u8; 3]]) -> u8 {
    let mut best_idx = 0u8;
    let mut best_dist = u32::MAX;

    for (i, color) in palette.iter().enumerate() {
        let dr = pixel[0] as i32 - color[0] as i32;
        let dg = pixel[1] as i32 - color[1] as i32;
        let db = pixel[2] as i32 - color[2] as i32;
        let dist = (dr * dr + dg * dg + db * db) as u32;

        if dist < best_dist {
            best_dist = dist;
            best_idx = i as u8;
        }
    }
    best_idx
}

pub fn png_to_gif_frame(
    png_path: &str,
    width: u16,
    height: u16,
) -> Result<Frame<'static>, Box<dyn std::error::Error>> {
    let img = ImageReader::open(png_path)?.decode()?.to_rgb8();
    let palette = get_color_palette();

    let mut indexed_pixels = Vec::with_capacity((width as usize) * (height as usize));
    for pixel in img.pixels() {
        let rgb = [pixel[0], pixel[1], pixel[2]];
        indexed_pixels.push(find_closest_color(&rgb, &palette));
    }

    Ok(Frame {
        width,
        height,
        buffer: std::borrow::Cow::Owned(indexed_pixels),
        delay: FRAME_DELAY,
        ..Frame::default()
    })
}
