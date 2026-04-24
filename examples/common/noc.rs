use plotters::prelude::*;
use std::collections::HashMap;

pub const N: usize = 5;
pub const PANEL_PX: u32 = 700;
pub const TITLE_PX: u32 = 120;
pub const LEGEND_PX: u32 = 100;
pub const FRAME_W: u32 = PANEL_PX * 2;
pub const FRAME_H: u32 = TITLE_PX + PANEL_PX + LEGEND_PX;

const TITLE_FONT: u32 = 38;
const SUB_FONT: u32 = 28;
const LABEL_FONT: u32 = 28;
const LEGEND_FONT: u32 = 22;
const PE_LABEL_FONT: u32 = 22;
const PACKET_R: i32 = 7;
const BG: RGBColor = RGBColor(20, 20, 30);
const PE: RGBColor = RGBColor(60, 60, 75);
const GRID: RGBColor = RGBColor(55, 55, 70);

pub type Loads = HashMap<(usize, usize), u32>;

pub struct Side<'a> {
    pub paths: &'a [Vec<usize>],
    pub loads: &'a Loads,
    pub current_path: Option<&'a Vec<usize>>,
    pub packets: &'a [(usize, usize, f64)],
}

#[inline]
pub fn idx(r: usize, c: usize) -> usize {
    r * N + c
}

#[inline]
pub fn rc(v: usize) -> (usize, usize) {
    (v / N, v % N)
}

fn heat(rho: f64) -> RGBColor {
    let t = rho.clamp(0.0, 1.0);
    RGBColor(
        (60.0 + t * 195.0) as u8,
        (160.0 - t * 100.0) as u8,
        (230.0 - t * 200.0) as u8,
    )
}

fn tile_xy(r: usize, c: usize, w: i32, h: i32) -> (i32, i32) {
    let m_top = 100;
    let m_bot = 80;
    let m_side = w / 14;
    let usable_w = w - 2 * m_side;
    let usable_h = h - m_top - m_bot;
    let sx = usable_w / (N as i32 - 1);
    let sy = usable_h / (N as i32 - 1);
    (m_side + c as i32 * sx, m_top + r as i32 * sy)
}

fn tile_size(w: i32) -> i32 {
    let m_side = w / 14;
    let sx = (w - 2 * m_side) / (N as i32 - 1);
    (sx as f32 * 0.42) as i32
}

fn perp(dr: i32, dc: i32, k: i32) -> (i32, i32) {
    if dr == 0 {
        (0, k * dc.signum())
    } else {
        (k * dr.signum(), 0)
    }
}

fn draw_panel<DB: DrawingBackend>(
    area: &DrawingArea<DB, plotters::coord::Shift>,
    label: &str,
    side: &Side,
    heat_max: u32,
) -> Result<(), Box<dyn std::error::Error>>
where
    DB::ErrorType: 'static,
{
    area.fill(&BG)?;
    let (wp, hp) = area.dim_in_pixel();
    let (w, h) = (wp as i32, hp as i32);
    let denom = heat_max.max(1) as f64;

    for r in 0..N {
        for c in 0..N {
            let v = idx(r, c);
            for &(dr, dc) in &[(0i32, 1i32), (1, 0), (0, -1), (-1, 0)] {
                let nr = r as i32 + dr;
                let nc = c as i32 + dc;
                if nr < 0 || nc < 0 || nr >= N as i32 || nc >= N as i32 {
                    continue;
                }
                let u = idx(nr as usize, nc as usize);
                let (x1, y1) = tile_xy(r, c, w, h);
                let (x2, y2) = tile_xy(nr as usize, nc as usize, w, h);
                let (ox, oy) = perp(dr, dc, 4);
                let load = *side.loads.get(&(v, u)).unwrap_or(&0);
                let (color, width) = if load == 0 {
                    (GRID, 2)
                } else {
                    (heat(load as f64 / denom), 5)
                };
                area.draw(&PathElement::new(
                    vec![(x1 + ox, y1 + oy), (x2 + ox, y2 + oy)],
                    color.stroke_width(width),
                ))?;
            }
        }
    }

    if let Some(path) = side.current_path
        && path.len() >= 2
    {
        for win in path.windows(2) {
            let (u, vn) = (win[0], win[1]);
            let (ur, uc) = rc(u);
            let (vr, vc) = rc(vn);
            let dr = vr as i32 - ur as i32;
            let dc = vc as i32 - uc as i32;
            let (x1, y1) = tile_xy(ur, uc, w, h);
            let (x2, y2) = tile_xy(vr, vc, w, h);
            let (ox, oy) = perp(dr, dc, 4);
            area.draw(&PathElement::new(
                vec![(x1 + ox, y1 + oy), (x2 + ox, y2 + oy)],
                YELLOW.stroke_width(6),
            ))?;
        }
        let (sr, sc) = rc(*path.first().unwrap());
        let (dr, dc) = rc(*path.last().unwrap());
        let (sx, sy) = tile_xy(sr, sc, w, h);
        let (dx, dy) = tile_xy(dr, dc, w, h);
        let s = tile_size(w);
        area.draw(&Circle::new((sx, sy), s / 2 + 6, GREEN.stroke_width(3)))?;
        area.draw(&Circle::new((dx, dy), s / 2 + 6, RED.stroke_width(3)))?;
    }

    for &(u, vn, frac) in side.packets {
        let (ur, uc) = rc(u);
        let (vr, vc) = rc(vn);
        let dr = vr as i32 - ur as i32;
        let dc = vc as i32 - uc as i32;
        let (x1, y1) = tile_xy(ur, uc, w, h);
        let (x2, y2) = tile_xy(vr, vc, w, h);
        let (ox, oy) = perp(dr, dc, 4);
        let mx = x1 + ((x2 - x1) as f64 * frac) as i32 + ox;
        let my = y1 + ((y2 - y1) as f64 * frac) as i32 + oy;
        area.draw(&Circle::new((mx, my), PACKET_R + 2, BG.filled()))?;
        area.draw(&Circle::new((mx, my), PACKET_R, WHITE.filled()))?;
    }

    let s = tile_size(w);
    let label_font = ("sans-serif", PE_LABEL_FONT)
        .into_font()
        .color(&WHITE.mix(0.65));
    for r in 0..N {
        for c in 0..N {
            let (x, y) = tile_xy(r, c, w, h);
            area.draw(&Rectangle::new(
                [(x - s / 2, y - s / 2), (x + s / 2, y + s / 2)],
                PE.filled(),
            ))?;
            if (r == 0 || r == N - 1) && (c == 0 || c == N - 1) {
                area.draw(&Text::new(
                    format!("PE({r},{c})"),
                    (x - 44, y + s / 2 + 6),
                    label_font.clone(),
                ))?;
            }
        }
    }

    area.draw(&Text::new(
        label.to_string(),
        (16, 12),
        ("sans-serif", LABEL_FONT).into_font().color(&WHITE),
    ))?;

    Ok(())
}

pub fn render_frame(
    rgb: &mut [u8],
    title: &str,
    subtitle: &str,
    xy: &Side,
    ad: &Side,
    heat_max: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::with_buffer(rgb, (FRAME_W, FRAME_H)).into_drawing_area();
    root.fill(&BG)?;

    let (title_area, rest) = root.split_vertically(TITLE_PX);
    let (body, legend_area) = rest.split_vertically(PANEL_PX);

    title_area.draw(&Text::new(
        title.to_string(),
        (20, 14),
        ("sans-serif", TITLE_FONT).into_font().color(&WHITE),
    ))?;
    title_area.draw(&Text::new(
        subtitle.to_string(),
        (20, 14 + TITLE_FONT as i32 + 8),
        ("sans-serif", SUB_FONT).into_font().color(&WHITE),
    ))?;

    let (left, right) = body.split_horizontally(PANEL_PX);
    draw_panel(&left, "XY routing (deterministic)", xy, heat_max)?;
    draw_panel(&right, "Adaptive Dijkstra (M/M/1)", ad, heat_max)?;

    draw_legend(&legend_area, heat_max)?;

    root.present()?;
    Ok(())
}

fn draw_legend<DB: DrawingBackend>(
    area: &DrawingArea<DB, plotters::coord::Shift>,
    heat_max: u32,
) -> Result<(), Box<dyn std::error::Error>>
where
    DB::ErrorType: 'static,
{
    let cy = (LEGEND_PX / 2) as i32;
    let font = ("sans-serif", LEGEND_FONT).into_font().color(&WHITE);

    let bar_x = 40;
    let bar_y = cy - 8;
    let bar_w = 320;
    let bar_h = 22;
    for px in 0..bar_w {
        let rho = px as f64 / bar_w as f64;
        area.draw(&Rectangle::new(
            [(bar_x + px, bar_y), (bar_x + px + 1, bar_y + bar_h)],
            heat(rho).filled(),
        ))?;
    }
    area.draw(&Text::new(
        "wire load (concurrent flows)",
        (bar_x, bar_y - 26),
        font.clone(),
    ))?;
    area.draw(&Text::new(
        "0",
        (bar_x - 6, bar_y + bar_h + 6),
        font.clone(),
    ))?;
    area.draw(&Text::new(
        format!("{heat_max}"),
        (bar_x + bar_w - 10, bar_y + bar_h + 6),
        font.clone(),
    ))?;

    let mid_x = bar_x + bar_w + 70;
    area.draw(&Circle::new((mid_x, cy), 12, GREEN.stroke_width(3)))?;
    area.draw(&Text::new("source PE", (mid_x + 18, cy - 12), font.clone()))?;
    area.draw(&Circle::new((mid_x + 170, cy), 12, RED.stroke_width(3)))?;
    area.draw(&Text::new("dest PE", (mid_x + 188, cy - 12), font.clone()))?;
    area.draw(&PathElement::new(
        vec![(mid_x + 320, cy), (mid_x + 380, cy)],
        YELLOW.stroke_width(6),
    ))?;
    area.draw(&Text::new(
        "allocated path",
        (mid_x + 388, cy - 12),
        font.clone(),
    ))?;
    let pkt_x = mid_x + 580;
    area.draw(&Circle::new((pkt_x, cy), PACKET_R + 2, BG.filled()))?;
    area.draw(&Circle::new((pkt_x, cy), PACKET_R, WHITE.filled()))?;
    area.draw(&Text::new("packet", (pkt_x + 16, cy - 12), font))?;

    Ok(())
}
