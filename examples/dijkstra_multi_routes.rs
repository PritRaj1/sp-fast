use std::fs::{File, create_dir_all};
use std::io::BufWriter;
use std::path::Path;

use gif::{Encoder, Frame, Repeat};
use sssp_fast::{AdjListGraph, Dijkstra, DijkstraConfig, Dyn, Graph, SsspAlgorithm, SsspBuffers};

const GRID: usize = 30;
const CELL: usize = 16;
const FRAME_DELAY_CS: u16 = 5;
const HOLD_FRAMES: usize = 25;
const OUT: &str = "examples/gifs/dijkstra_multi_routes.gif";

const C_UNREVEALED: u8 = 0;
const C_PAIR_BASE: u8 = 1;
const C_RING: u8 = 6;

const PALETTE: [[u8; 3]; 7] = [
    [40, 40, 50],
    [230, 80, 90],
    [80, 180, 230],
    [90, 220, 120],
    [230, 180, 60],
    [180, 100, 230],
    [255, 255, 255],
];

type Coord = (usize, usize);

#[inline]
fn idx(r: usize, c: usize) -> usize {
    r * GRID + c
}

fn build_grid() -> AdjListGraph<f64> {
    let n = GRID * GRID;
    let mut g = AdjListGraph::new(n);
    for r in 0..GRID {
        for c in 0..GRID {
            let v = idx(r, c);
            if c + 1 < GRID {
                g.add_edge(v, v + 1, 1.0);
                g.add_edge(v + 1, v, 1.0);
            }
            if r + 1 < GRID {
                g.add_edge(v, v + GRID, 1.0);
                g.add_edge(v + GRID, v, 1.0);
            }
        }
    }
    g
}

fn paint_cell(pixels: &mut [u8], stride: usize, r: usize, c: usize, color: u8) {
    for dy in 0..CELL {
        let row = (r * CELL + dy) * stride + c * CELL;
        pixels[row..row + CELL].fill(color);
    }
}

fn paint_ring(pixels: &mut [u8], stride: usize, r: usize, c: usize, color: u8) {
    let outer = (CELL / 3) as i32;
    let inner = (CELL / 5) as i32;
    let (outer_sq, inner_sq) = (outer * outer, inner * inner);
    let cy = (r * CELL + CELL / 2) as i32;
    let cx = (c * CELL + CELL / 2) as i32;
    for dy in -outer..=outer {
        for dx in -outer..=outer {
            let d2 = dx * dx + dy * dy;
            if (inner_sq..=outer_sq).contains(&d2) {
                let y = cy + dy;
                let x = cx + dx;
                if y >= 0 && x >= 0 && (y as usize) < GRID * CELL && (x as usize) < stride {
                    pixels[y as usize * stride + x as usize] = color;
                }
            }
        }
    }
}

fn write_frame(
    enc: &mut Encoder<BufWriter<File>>,
    width: u16,
    height: u16,
    pixels: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let frame = Frame {
        width,
        height,
        buffer: pixels.into(),
        delay: FRAME_DELAY_CS,
        ..Frame::default()
    };
    enc.write_frame(&frame)?;
    Ok(())
}

fn route(g: &AdjListGraph<f64>, src: usize, tgt: usize, n: usize) -> Vec<Coord> {
    let mut buf: SsspBuffers<f64, Dyn> = SsspBuffers::new_inf(Dyn(n));
    let mut algo: Dijkstra<f64> = Dijkstra::with_config(DijkstraConfig::with_target(tgt));
    algo.run(g, src, &mut buf);
    buf.path_to(tgt)
        .unwrap_or_default()
        .into_iter()
        .map(|v| (v / GRID, v % GRID))
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let g = build_grid();
    let n = g.n();

    let pairs: [(Coord, Coord); 5] = [
        ((0, 0), (29, 29)),
        ((0, 29), (29, 0)),
        ((15, 0), (15, 29)),
        ((0, 15), (29, 15)),
        ((5, 25), (25, 5)),
    ];

    let routes: Vec<Vec<Coord>> = pairs
        .iter()
        .map(|&(s, t)| route(&g, idx(s.0, s.1), idx(t.0, t.1), n))
        .collect();

    let stride = GRID * CELL;
    let height = GRID * CELL;
    let mut pixels = vec![C_UNREVEALED; stride * height];

    for (i, &(s, _)) in pairs.iter().enumerate() {
        paint_cell(&mut pixels, stride, s.0, s.1, C_PAIR_BASE + i as u8);
    }

    if let Some(dir) = Path::new(OUT).parent() {
        create_dir_all(dir)?;
    }
    let flat: Vec<u8> = PALETTE.iter().flatten().copied().collect();
    let mut enc = Encoder::new(
        BufWriter::new(File::create(OUT)?),
        stride as u16,
        height as u16,
        &flat,
    )?;
    enc.set_repeat(Repeat::Infinite)?;

    // Rings redrawn every frame so path-cells don't overwrite them.
    let emit = |enc: &mut Encoder<BufWriter<File>>,
                pixels: &mut [u8]|
     -> Result<(), Box<dyn std::error::Error>> {
        for &(_, t) in &pairs {
            paint_ring(pixels, stride, t.0, t.1, C_RING);
        }
        write_frame(enc, stride as u16, height as u16, pixels)
    };

    emit(&mut enc, &mut pixels)?;

    let max_len = routes.iter().map(Vec::len).max().unwrap_or(0);
    for step in 1..max_len {
        for (i, path) in routes.iter().enumerate() {
            if let Some(&(r, c)) = path.get(step) {
                paint_cell(&mut pixels, stride, r, c, C_PAIR_BASE + i as u8);
            }
        }
        emit(&mut enc, &mut pixels)?;
    }

    for _ in 0..HOLD_FRAMES {
        emit(&mut enc, &mut pixels)?;
    }

    println!("wrote {}", OUT);
    Ok(())
}
