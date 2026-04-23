/// Floyd-Warshall config. `detect_negative_cycle` costs one diagonal scan.
#[derive(Clone, Debug)]
pub struct FloydWarshallConfig {
    pub detect_negative_cycle: bool,
}

impl Default for FloydWarshallConfig {
    fn default() -> Self {
        Self {
            detect_negative_cycle: true,
        }
    }
}

impl FloydWarshallConfig {
    pub fn without_negative_cycle_detection(mut self) -> Self {
        self.detect_negative_cycle = false;
        self
    }
}
