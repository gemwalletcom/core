pub enum DomainResolution {
    Host(primitives::Chain),
    Path(primitives::Chain),
}

impl DomainResolution {
    pub fn chain(&self) -> primitives::Chain {
        match self {
            Self::Host(chain) | Self::Path(chain) => *chain,
        }
    }

    pub fn is_path_based(&self) -> bool {
        match self {
            Self::Path(_) => true,
            Self::Host(_) => false,
        }
    }
}
