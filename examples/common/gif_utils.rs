use gif::{Encoder, Frame, Repeat};
use image::ImageReader;
use std::fs::File;
use std::io::BufWriter;

use super::state::visited_gradient_rgb;

pub const FRAME_DELAY: u16 = 4;

/// Every RGB plotters paints + 64-step visit gradient; padded to 256 entries.
fn palette() -> Vec<[u8; 3]> {
    let mut p = Vec::with_capacity(256);
    p.push([10, 10, 10]); // background
    p.push([255, 255, 255]); // text, highlight outline
    p.push([80, 80, 80]); // unvisited node
    p.push([255, 200, 50]); // in-queue node
    p.push([100, 180, 255]); // visited node (pre-gradient)
    p.push([50, 255, 100]); // path
    p.push([255, 80, 80]); // start
    p.push([180, 50, 255]); // end
    p.push([50, 50, 50]); // default edge
    p.push([80, 120, 160]); // relaxed edge
    for i in 0..64 {
        let (r, g, b) = visited_gradient_rgb(i, 63);
        p.push([r, g, b]);
    }
    p.resize(256, [0, 0, 0]);
    p
}

pub fn setup_gif(
    path: &str,
    width: u16,
    height: u16,
) -> Result<Encoder<BufWriter<File>>, Box<dyn std::error::Error>> {
    let flat: Vec<u8> = palette().iter().flatten().copied().collect();
    let writer = BufWriter::new(File::create(path)?);
    let mut encoder = Encoder::new(writer, width, height, &flat)?;
    encoder.set_repeat(Repeat::Infinite)?;
    Ok(encoder)
}

fn find_closest(pixel: &[u8; 3], palette: &[[u8; 3]]) -> u8 {
    palette
        .iter()
        .enumerate()
        .min_by_key(|(_, c)| {
            let dr = pixel[0] as i32 - c[0] as i32;
            let dg = pixel[1] as i32 - c[1] as i32;
            let db = pixel[2] as i32 - c[2] as i32;
            dr * dr + dg * dg + db * db
        })
        .map(|(i, _)| i as u8)
        .unwrap_or(0)
}

pub fn png_to_gif_frame(
    png_path: &str,
    width: u16,
    height: u16,
) -> Result<Frame<'static>, Box<dyn std::error::Error>> {
    let img = ImageReader::open(png_path)?.decode()?.to_rgb8();
    let pal = palette();
    let indexed: Vec<u8> = img
        .pixels()
        .map(|p| find_closest(&[p[0], p[1], p[2]], &pal))
        .collect();

    Ok(Frame {
        width,
        height,
        buffer: std::borrow::Cow::Owned(indexed),
        delay: FRAME_DELAY,
        ..Frame::default()
    })
}
