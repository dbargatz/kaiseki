use async_trait::async_trait;
use uuid::Uuid;

pub type ComponentId = Uuid;

#[async_trait]
pub trait Component: 'static + Send + Sync {
    fn id(&self) -> ComponentId;
    async fn start(&mut self);
}
