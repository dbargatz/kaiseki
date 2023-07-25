use std::fmt;

use anyhow::Result;
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

pub type Component2Id = usize;

pub trait Component: 'static + Send + Sync {
    fn id(&self) -> &ComponentId;
}

impl PartialEq for dyn Component + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
impl Eq for dyn Component + '_ {}

pub trait AddressableComponent: Component {
    fn read(&self, address: usize, length: usize) -> Result<Vec<u8>>;
    fn write(&self, address: usize, data: &[u8]) -> Result<()>;
}

impl PartialEq for dyn AddressableComponent + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
impl Eq for dyn AddressableComponent + '_ {}

#[async_trait]
pub trait ExecutableComponent: Component {
    async fn start(&self);
}

#[async_trait]
pub trait Component2 {
    type Ref;

    fn id(&self) -> &Component2Id;
    async fn run(&mut self);
}

#[derive(Debug)]
pub struct Component2Ref<T> {
    id: Component2Id,
    ref_receiver: async_channel::Receiver<T>,
    ref_sender: async_channel::Sender<T>,
}

impl<T> Component2Ref<T> {
    pub fn id(&self) -> &Component2Id {
        &self.id
    }

    pub fn new(
        id: &Component2Id,
        ref_receiver: async_channel::Receiver<T>,
        ref_sender: async_channel::Sender<T>,
    ) -> Self {
        Self {
            id: *id,
            ref_receiver,
            ref_sender,
        }
    }

    pub fn send_blocking(&self, msg: T) {
        self.ref_sender.send_blocking(msg).unwrap()
    }

    pub async fn send(&self, msg: T) {
        self.ref_sender.send(msg).await.unwrap()
    }

    pub async fn recv(&self) -> T {
        self.ref_receiver.recv().await.unwrap()
    }
}
