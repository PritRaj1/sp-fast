use std::fs;

use super::gif_utils::{png_to_gif_frame, setup_gif};
use super::rendering::{
    GRAPH_HEIGHT, GRAPH_WIDTH, GraphRenderParams, TITLE_HEIGHT, render_graph_frame,
};
use super::vis::GraphVisState;

const TMP_PNG: &str = "examples/gifs/_tmp.png";

/// Encode sequence of graph frames as GIF.
pub fn write_graph_gif(
    out: &str,
    frames: &[GraphVisState],
    titles: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let width = GRAPH_WIDTH as u16;
    let height = (GRAPH_HEIGHT + TITLE_HEIGHT) as u16;
    let mut enc = setup_gif(out, width, height)?;
    for (i, frame) in frames.iter().enumerate() {
        let title = titles.get(i).map(String::as_str).unwrap_or("");
        render_graph_frame(
            TMP_PNG,
            GraphRenderParams {
                state: frame,
                title,
            },
        )?;
        enc.write_frame(&png_to_gif_frame(TMP_PNG, width, height)?)?;
    }
    fs::remove_file(TMP_PNG).ok();
    Ok(())
}

/// First `explore_count` titles from `explore_title(i)`; rest from `final_title`.
pub fn make_titles(
    n_frames: usize,
    explore_count: usize,
    explore_title: impl Fn(usize) -> String,
    final_title: String,
) -> Vec<String> {
    (0..n_frames)
        .map(|i| {
            if i < explore_count {
                explore_title(i)
            } else {
                final_title.clone()
            }
        })
        .collect()
}
