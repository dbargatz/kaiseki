use std::fmt::Debug;
use std::sync::mpsc;

use crate::ActorState;

#[derive(Debug)]
pub struct ThreadActor<T: ActorState> {
    state: T,
}

impl<T: ActorState> ThreadActor<T> {
    pub fn create(state: T) -> ThreadActorHandle<T> {
        // TODO: how should this channel be bounded?
        let (tx, rx) = mpsc::sync_channel(1);
        let mut actor = ThreadActor { state };

        // TODO: where should the JoinHandle for this task be stored, if anywhere?
        std::thread::spawn(move || {
            tracing::info!("starting ThreadActor");
            while let Ok(message) = rx.recv() {
                actor.state.dispatch(message)
            }
            tracing::info!("exiting ThreadActor");
        });
        ThreadActorHandle { sender: tx }
    }
}

#[derive(Debug)]
pub struct ThreadActorHandle<T: ActorState> {
    sender: mpsc::SyncSender<T::Message>,
}

impl<T: ActorState> Clone for ThreadActorHandle<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<T: ActorState> ThreadActorHandle<T> {
    pub fn send_message(&self, message: T::Message) {
        self.sender
            .send(message)
            .expect("request sent successfully");
    }
}

#[cfg(test)]
mod tests {
    use super::{ThreadActor, ThreadActorHandle};
    use crate::mocks::{TestMessage, Tester};
    use futures::channel::oneshot;

    pub type TestActor = ThreadActor<Tester>;
    pub type TestActorHandle = ThreadActorHandle<Tester>;
    impl TestActorHandle {
        pub fn add(&self, num: usize) -> usize {
            let (tx, rx) = oneshot::channel::<usize>();
            self.sender
                .send(TestMessage::Add { num, response: tx })
                .expect("add request sent successfully");
            futures::executor::block_on(rx).expect("add response received successfully")
        }

        pub fn get_sum(&self) -> usize {
            let (tx, rx) = oneshot::channel::<usize>();
            self.sender
                .send(TestMessage::GetSum { response: tx })
                .expect("add request sent successfully");
            futures::executor::block_on(rx).expect("add response received successfully")
        }
    }

    #[test]
    fn can_create_new_actor() {
        let state = Tester::new(0);
        let actor = TestActor::create(state);
        assert!(actor.get_sum() == 0, "incorrect value for get_sum()");
    }

    #[test]
    fn can_clone_actor_handle() {
        let state = Tester::new(0);
        let actor1: TestActorHandle = TestActor::create(state);
        let actor2 = actor1.clone();

        // Ensure the actor1 and actor2 handles refer to the same actor.
        assert!(
            actor1.get_sum() == actor2.get_sum(),
            "initial sums do not match"
        );
        let sum1 = actor1.add(7);
        assert!(sum1 == actor1.get_sum(), "sum1 and actor1 sum do not match");
        let sum2 = actor2.add(5);
        assert!(sum2 == actor2.get_sum(), "sum2 and actor2 sum do not match");
        assert!(
            actor1.get_sum() == actor2.get_sum(),
            "new sums do not match"
        );
    }
}
