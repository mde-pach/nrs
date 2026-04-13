/// NRS context layer — the core organizational concept.
///
/// Layers form a hierarchy from outermost (most abstract, widest scope)
/// to innermost (most concrete, narrowest scope).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Nrs,
    Corporate,
    Team,
    Project,
    Domain,
    Implementation,
    Custom,
}

impl Layer {
    /// Derive the layer from a context filename.
    pub fn from_filename(name: &str) -> Self {
        match name {
            "nrs.context.md" => Self::Nrs,
            "corporate.context.md" => Self::Corporate,
            "team.context.md" => Self::Team,
            "project.context.md" => Self::Project,
            "domain.context.md" => Self::Domain,
            "implementation.context.md" => Self::Implementation,
            _ => Self::Custom,
        }
    }

    /// Sort priority for generated output (lower = earlier).
    pub fn sort_priority(self) -> u8 {
        match self {
            Self::Nrs => 0,
            Self::Corporate => 1,
            Self::Team => 2,
            Self::Project => 3,
            Self::Domain => 4,
            Self::Implementation => 5,
            Self::Custom => 6,
        }
    }

    /// Whether this is a root-level context (always-loaded).
    pub fn is_root(self) -> bool {
        matches!(
            self,
            Self::Nrs | Self::Corporate | Self::Team | Self::Project
        )
    }

    /// Maximum recommended line count for this layer.
    pub fn size_limit(self) -> usize {
        if self.is_root() {
            500
        } else {
            300
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_filename_known_layers() {
        assert_eq!(Layer::from_filename("nrs.context.md"), Layer::Nrs);
        assert_eq!(
            Layer::from_filename("corporate.context.md"),
            Layer::Corporate
        );
        assert_eq!(Layer::from_filename("team.context.md"), Layer::Team);
        assert_eq!(Layer::from_filename("project.context.md"), Layer::Project);
        assert_eq!(Layer::from_filename("domain.context.md"), Layer::Domain);
        assert_eq!(
            Layer::from_filename("implementation.context.md"),
            Layer::Implementation
        );
    }

    #[test]
    fn from_filename_unknown_is_custom() {
        assert_eq!(Layer::from_filename("custom.context.md"), Layer::Custom);
        assert_eq!(Layer::from_filename("anything.context.md"), Layer::Custom);
    }

    #[test]
    fn sort_priority_ordering() {
        assert!(Layer::Nrs.sort_priority() < Layer::Corporate.sort_priority());
        assert!(Layer::Corporate.sort_priority() < Layer::Team.sort_priority());
        assert!(Layer::Team.sort_priority() < Layer::Project.sort_priority());
        assert!(Layer::Project.sort_priority() < Layer::Domain.sort_priority());
        assert!(Layer::Domain.sort_priority() < Layer::Implementation.sort_priority());
        assert!(Layer::Implementation.sort_priority() < Layer::Custom.sort_priority());
    }

    #[test]
    fn root_layers() {
        assert!(Layer::Nrs.is_root());
        assert!(Layer::Corporate.is_root());
        assert!(Layer::Team.is_root());
        assert!(Layer::Project.is_root());
        assert!(!Layer::Domain.is_root());
        assert!(!Layer::Implementation.is_root());
        assert!(!Layer::Custom.is_root());
    }

    #[test]
    fn size_limits() {
        assert_eq!(Layer::Nrs.size_limit(), 500);
        assert_eq!(Layer::Project.size_limit(), 500);
        assert_eq!(Layer::Domain.size_limit(), 300);
        assert_eq!(Layer::Implementation.size_limit(), 300);
        assert_eq!(Layer::Custom.size_limit(), 300);
    }
}
