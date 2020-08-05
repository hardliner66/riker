use std::{
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

pub struct ActorPath(Arc<String>);

impl ActorPath {
    #[cfg_attr(feature = "profiling", optick_attr::profile)]
    pub fn new(path: &str) -> Self {
        ActorPath(Arc::new(path.to_string()))
    }
}

impl PartialEq for ActorPath {
    #[cfg_attr(feature = "profiling", optick_attr::profile)]
    fn eq(&self, other: &ActorPath) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<str> for ActorPath {
    #[cfg_attr(feature = "profiling", optick_attr::profile)]
    fn eq(&self, other: &str) -> bool {
        *self.0 == other
    }
}

impl Eq for ActorPath {}

impl Hash for ActorPath {
    #[cfg_attr(feature = "profiling", optick_attr::profile)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl fmt::Display for ActorPath {
    #[cfg_attr(feature = "profiling", optick_attr::profile)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for ActorPath {
    #[cfg_attr(feature = "profiling", optick_attr::profile)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Clone for ActorPath {
    #[cfg_attr(feature = "profiling", optick_attr::profile)]
    fn clone(&self) -> Self {
        ActorPath(self.0.clone())
    }
}

/// An `ActorUri` represents the location of an actor, including the
/// path and actor system host.
///
/// Note: `host` is currently unused but will be utilized when
/// networking and clustering are introduced.
#[derive(Clone)]
pub struct ActorUri {
    pub name: Arc<String>,
    pub path: ActorPath,
    pub host: Arc<String>,
}

impl PartialEq for ActorUri {
    #[cfg_attr(feature = "profiling", optick_attr::profile)]
    fn eq(&self, other: &ActorUri) -> bool {
        self.path == other.path
    }
}

impl Eq for ActorUri {}

impl Hash for ActorUri {
    #[cfg_attr(feature = "profiling", optick_attr::profile)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

impl fmt::Display for ActorUri {
    #[cfg_attr(feature = "profiling", optick_attr::profile)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl fmt::Debug for ActorUri {
    #[cfg_attr(feature = "profiling", optick_attr::profile)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}://{}", self.host, self.path)
    }
}
