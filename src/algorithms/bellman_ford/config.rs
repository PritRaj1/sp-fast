use crate::algorithms::{HasSsspConfig, SsspConfig};

/// Bellman-Ford config. `early_termination` = stop when iter makes no progress.
#[derive(Clone, Debug)]
pub struct BellmanFordConfig {
    base: SsspConfig,
    pub early_termination: bool,
}

impl Default for BellmanFordConfig {
    fn default() -> Self {
        Self {
            base: SsspConfig::default(),
            early_termination: true,
        }
    }
}

impl BellmanFordConfig {
    pub fn with_target(target: usize) -> Self {
        Self::with_targets(vec![target])
    }

    pub fn with_targets(targets: Vec<usize>) -> Self {
        Self {
            base: SsspConfig::with_targets(targets),
            early_termination: true,
        }
    }

    pub fn without_early_termination(mut self) -> Self {
        self.early_termination = false;
        self
    }
}

impl HasSsspConfig for BellmanFordConfig {
    fn sssp_config(&self) -> &SsspConfig {
        &self.base
    }
}
