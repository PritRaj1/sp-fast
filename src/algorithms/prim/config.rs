/// Prim config. `lazy_deletion` = skip stale heap entries.
#[derive(Clone, Debug)]
pub struct PrimConfig {
    pub lazy_deletion: bool,
}

impl Default for PrimConfig {
    fn default() -> Self {
        Self {
            lazy_deletion: true,
        }
    }
}

impl PrimConfig {
    pub fn without_lazy_deletion(mut self) -> Self {
        self.lazy_deletion = false;
        self
    }
}
