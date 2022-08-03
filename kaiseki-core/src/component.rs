use std::fmt;

use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct ComponentId {
    name: String,
    uuid: Uuid,
}

impl Clone for ComponentId {
    fn clone(&self) -> Self {
        Self {
            name: String::from(self.name.as_str()),
            uuid: self.uuid,
        }
    }
}

impl fmt::Display for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name.as_str())
    }
}

impl ComponentId {
    pub fn new(name: &str) -> Self {
        ComponentId {
            name: String::from(name),
            uuid: Uuid::new_v4(),
        }
    }
}

pub trait Component: 'static + Send + Sync {
    fn id(&self) -> ComponentId;
}

#[async_trait]
pub trait ExecutableComponent: Component {
    async fn start(&mut self);
}
