/// Dijkstra config. `targets` enable multi-target early-stop;
#[derive(Clone, Debug)]
pub struct DijkstraConfig {
    pub targets: Vec<usize>,
    pub lazy_deletion: bool,
}

impl Default for DijkstraConfig {
    fn default() -> Self {
        Self {
            targets: Vec::new(),
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
            targets,
            lazy_deletion: true,
        }
    }

    pub fn without_lazy_deletion(mut self) -> Self {
        self.lazy_deletion = false;
        self
    }

    pub fn set_target(&mut self, target: usize) {
        self.targets.clear();
        self.targets.push(target);
    }

    pub fn set_targets(&mut self, targets: Vec<usize>) {
        self.targets = targets;
    }
}
