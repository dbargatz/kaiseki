#[cfg(test)]
mod mocks;
mod thread;
#[cfg(feature = "tokio")]
mod tokio;

#[cfg(feature = "tokio")]
pub use crate::tokio::{TokioActor, TokioActorHandle};
pub use thread::{ThreadActor, ThreadActorHandle};

pub trait ActorState: Send + 'static {
    type Message: Send;

    fn dispatch(&mut self, message: Self::Message);
}
