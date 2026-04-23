/// Bellman-Ford config. `early_termination` = stop when iter makes no progress.
#[derive(Clone, Debug)]
pub struct BellmanFordConfig {
    pub early_termination: bool,
}

impl Default for BellmanFordConfig {
    fn default() -> Self {
        Self {
            early_termination: true,
        }
    }
}

impl BellmanFordConfig {
    pub fn without_early_termination(mut self) -> Self {
        self.early_termination = false;
        self
    }
}
