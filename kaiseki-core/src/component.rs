use uuid::Uuid;

pub type ComponentId = Uuid;

pub trait Component {
    fn id(&self) -> ComponentId;
    fn start(&self);
}
