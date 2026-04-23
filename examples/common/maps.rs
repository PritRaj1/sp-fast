use sssp_fast::Graph;

/// 2D grid with walls and walkable cells.
#[derive(Clone)]
pub struct GridMap {
    pub rows: usize,
    pub cols: usize,
    pub walls: Vec<Vec<bool>>,
}

/// 4-connected grid with unit-weight edges between walkable neighbours.
/// Walls participate as vertices with zero out-edges (keeps vertex IDs stable
/// against `to_vertex(row, col) = row * cols + col`).
impl Graph<f64> for GridMap {
    type Meta = ();
    fn n(&self) -> usize {
        self.rows * self.cols
    }
    fn for_each_out_edge<F: FnMut(usize, f64, &())>(&self, u: usize, mut f: F) {
        let (r, c) = (u / self.cols, u % self.cols);
        if self.is_wall(r, c) {
            return;
        }
        let mut emit = |nr: usize, nc: usize| {
            if !self.is_wall(nr, nc) {
                f(nr * self.cols + nc, 1.0, &());
            }
        };
        if r > 0 {
            emit(r - 1, c);
        }
        if r + 1 < self.rows {
            emit(r + 1, c);
        }
        if c > 0 {
            emit(r, c - 1);
        }
        if c + 1 < self.cols {
            emit(r, c + 1);
        }
    }
}

impl GridMap {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            walls: vec![vec![false; cols]; rows],
        }
    }

    pub fn set_wall(&mut self, row: usize, col: usize) {
        if row < self.rows && col < self.cols {
            self.walls[row][col] = true;
        }
    }

    pub fn is_wall(&self, row: usize, col: usize) -> bool {
        row >= self.rows || col >= self.cols || self.walls[row][col]
    }

    pub fn is_passable(&self, row: usize, col: usize) -> bool {
        !self.is_wall(row, col)
    }

    pub fn to_vertex(&self, row: usize, col: usize) -> usize {
        row * self.cols + col
    }

    pub fn to_coords(&self, vertex: usize) -> (usize, usize) {
        (vertex / self.cols, vertex % self.cols)
    }
}

pub fn maze() -> (GridMap, (usize, usize), (usize, usize)) {
    let mut map = GridMap::new(18, 24);

    // Outer border
    for col in 0..24 {
        map.set_wall(0, col);
        map.set_wall(17, col);
    }
    for row in 0..18 {
        map.set_wall(row, 0);
        map.set_wall(row, 23);
    }

    // Horizontal walls
    for col in 2..10 {
        map.set_wall(4, col);
    }
    for col in 14..22 {
        map.set_wall(4, col);
    }
    for col in 5..15 {
        map.set_wall(8, col);
    }
    for col in 2..12 {
        map.set_wall(12, col);
    }
    for col in 16..22 {
        map.set_wall(12, col);
    }

    // Vertical walls
    for row in 4..10 {
        map.set_wall(row, 12);
    }
    for row in 8..15 {
        map.set_wall(row, 6);
    }
    for row in 12..17 {
        map.set_wall(row, 18);
    }
    for row in 1..6 {
        map.set_wall(row, 18);
    }

    let start = (1, 1);
    let end = (16, 22);

    (map, start, end)
}
