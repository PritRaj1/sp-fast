/// Floyd-Warshall config. `detect_negative_cycle` costs one diagonal scan.
#[derive(Clone, Debug, Default)]
pub struct FloydWarshallConfig {
    pub detect_negative_cycle: bool,
}

impl FloydWarshallConfig {
    pub fn new() -> Self {
        Self {
            detect_negative_cycle: true,
        }
    }

    pub fn without_negative_cycle_detection(mut self) -> Self {
        self.detect_negative_cycle = false;
        self
    }
}
