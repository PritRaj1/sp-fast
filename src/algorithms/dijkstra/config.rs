use crate::algorithms::{HasSsspConfig, SsspConfig};

/// Dijkstra config. `lazy_deletion` = skip stale heap entries.
#[derive(Clone, Debug)]
pub struct DijkstraConfig {
    base: SsspConfig,
    pub lazy_deletion: bool,
}

impl Default for DijkstraConfig {
    fn default() -> Self {
        Self {
            base: SsspConfig::default(),
            lazy_deletion: true,
        }
    }
}

impl DijkstraConfig {
    pub fn with_target(target: usize) -> Self {
        Self::with_targets(vec![target])
    }

    pub fn with_targets(targets: Vec<usize>) -> Self {
        Self {
            base: SsspConfig::with_targets(targets),
            lazy_deletion: true,
        }
    }

    pub fn without_lazy_deletion(mut self) -> Self {
        self.lazy_deletion = false;
        self
    }
}

impl HasSsspConfig for DijkstraConfig {
    fn sssp_config(&self) -> &SsspConfig {
        &self.base
    }
}
