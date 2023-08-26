use std::fmt::Debug;

use tokio::sync::mpsc;

use crate::ActorState;

#[derive(Debug)]
pub struct TokioActor<T: ActorState> {
    state: T,
}

impl<T: ActorState> TokioActor<T> {
    pub fn create(state: T) -> TokioActorHandle<T> {
        // TODO: how should this channel be bounded?
        let (tx, mut rx) = mpsc::channel(1);
        let mut actor = TokioActor { state };

        // TODO: where should the JoinHandle for this task be stored, if anywhere?
        tokio::spawn(async move {
            tracing::info!("starting TokioActor");
            while let Some(message) = rx.recv().await {
                actor.state.dispatch(message)
            }
            tracing::info!("exiting TokioActor");
        });
        TokioActorHandle { sender: tx }
    }
}

#[derive(Debug)]
pub struct TokioActorHandle<T: ActorState> {
    sender: mpsc::Sender<T::Message>,
}

impl<T: ActorState> Clone for TokioActorHandle<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<T: ActorState> TokioActorHandle<T> {
    pub async fn send_message(&self, message: T::Message) {
        self.sender
            .send(message)
            .await
            .expect("request sent successfully");
    }
}

#[cfg(test)]
mod tests {
    use super::{TokioActor, TokioActorHandle};
    use crate::mocks::{TestMessage, Tester};
    use futures::channel::oneshot;

    pub type TestActor = TokioActor<Tester>;
    pub type TestActorHandle = TokioActorHandle<Tester>;
    impl TestActorHandle {
        pub async fn add(&self, num: usize) -> usize {
            let (tx, rx) = oneshot::channel::<usize>();
            self.sender
                .send(TestMessage::Add { num, response: tx })
                .await
                .expect("add request sent successfully");
            rx.await.expect("add response received successfully")
        }

        pub async fn get_sum(&self) -> usize {
            let (tx, rx) = oneshot::channel::<usize>();
            self.sender
                .send(TestMessage::GetSum { response: tx })
                .await
                .expect("add request sent successfully");
            rx.await.expect("add response received successfully")
        }
    }

    #[tokio::test]
    async fn can_create_new_actor() {
        let state = Tester::new(0);
        let actor = TestActor::create(state);
        assert!(actor.get_sum().await == 0, "incorrect value for get_sum()");
    }

    #[tokio::test]
    async fn can_clone_actor_handle() {
        let state = Tester::new(0);
        let actor1: TestActorHandle = TestActor::create(state);
        let actor2 = actor1.clone();

        // Ensure the actor1 and actor2 handles refer to the same actor.
        assert!(
            actor1.get_sum().await == actor2.get_sum().await,
            "initial sums do not match"
        );
        let sum1 = actor1.add(7).await;
        assert!(
            sum1 == actor1.get_sum().await,
            "sum1 and actor1 sum do not match"
        );
        let sum2 = actor2.add(5).await;
        assert!(
            sum2 == actor2.get_sum().await,
            "sum2 and actor2 sum do not match"
        );
        assert!(
            actor1.get_sum().await == actor2.get_sum().await,
            "new sums do not match"
        );
    }
}

// #[async_trait]
// pub trait Component2 {
//     type Ref;

//     fn id(&self) -> &Component2Id;
//     async fn run(&mut self);
// }

// #[derive(Debug)]
// pub struct Component2Ref<T> {
//     id: Component2Id,
//     ref_receiver: async_channel::Receiver<T>,
//     ref_sender: async_channel::Sender<T>,
// }

// impl<T> Component2Ref<T> {
//     pub fn id(&self) -> &Component2Id {
//         &self.id
//     }

//     pub fn new(
//         id: &Component2Id,
//         ref_receiver: async_channel::Receiver<T>,
//         ref_sender: async_channel::Sender<T>,
//     ) -> Self {
//         Self {
//             id: *id,
//             ref_receiver,
//             ref_sender,
//         }
//     }

//     pub fn send_blocking(&self, msg: T) {
//         self.ref_sender.send_blocking(msg).unwrap()
//     }

//     pub async fn send(&self, msg: T) {
//         self.ref_sender.send(msg).await.unwrap()
//     }

//     pub async fn recv(&self) -> T {
//         self.ref_receiver.recv().await.unwrap()
//     }
// }

// #[derive(Debug)]
// struct ClockMetrics {
//     start_time: Instant,
//     stop_time: Option<Instant>,
// }

// impl ClockMetrics {
//     pub fn new() -> Self {
//         Self {
//             start_time: Instant::now(),
//             stop_time: None,
//         }
//     }
// }

// pub enum Clock2Message {
//     RunCycles(usize),
//     StartClock,
//     StopClock,
// }

// #[derive(Debug)]
// pub struct Clock2 {
//     id: Component2Id,
//     bus: ClockBusRef,
//     current_cycle: usize,
//     frequency_hz: usize,
//     metrics: ClockMetrics,
//     multiplier: f64,
//     ref_receiver: async_channel::Receiver<Clock2Message>,
//     running: bool,
// }

// #[async_trait]
// impl Component2 for Clock2 {
//     type Ref = Clock2Ref;

//     fn id(&self) -> &Component2Id {
//         &self.id
//     }

//     async fn run(&mut self) {
//         loop {
//             let period_secs = (1.0 / self.frequency_hz as f64) * self.multiplier;
//             let period = Duration::from_secs_f64(period_secs);
//             let sleep_fut = tokio::time::sleep(period);

//             let ref_receiver = self.ref_receiver.clone();
//             let receiver_fut = ref_receiver.recv();

//             tokio::select! {
//                 res = receiver_fut, if !ref_receiver.is_closed() => {
//                     match res {
//                         Ok(Clock2Message::RunCycles(num_cycles)) => {
//                             if self.running {
//                                 tracing::warn!("cannot run cycles, clock already running");
//                                 continue
//                             }
//                             self.running = true;
//                             self.metrics.start_time = Instant::now();
//                             self.metrics.stop_time = None;
//                             for i in 0..num_cycles {
//                                 self.bus.start_cycle(i).await;
//                                 tokio::time::sleep(period).await;
//                             }
//                             self.running = false;
//                             self.metrics.stop_time = Some(Instant::now());
//                             let duration = Instant::now() - self.metrics.start_time;
//                             tracing::info!("took {:0.5}seconds to run {} cycles", duration.as_secs_f32(), num_cycles);
//                         },
//                         Ok(Clock2Message::StartClock) => {
//                             if self.running {
//                                 tracing::warn!("cannot start clock, already running");
//                                 continue
//                             }
//                             self.running = true;
//                             self.metrics.start_time = Instant::now();
//                             self.metrics.stop_time = None;
//                         },
//                         Ok(Clock2Message::StopClock) => {
//                             if !self.running {
//                                 tracing::warn!("cannot stop clock, is not running");
//                                 continue
//                             }
//                             self.running = false;
//                             self.metrics.stop_time = Some(Instant::now());
//                         },
//                         Err(err) => {
//                             tracing::error!("receive error for open receiver: {}", err);
//                         }
//                     }
//                 },
//                 _ = sleep_fut, if self.running => {
//                     self.current_cycle += 1;
//                     self.bus.start_cycle(self.current_cycle).await;
//                 },
//                 else => {
//                     tracing::info!("component {} (clock) exiting", self.id());
//                     self.bus.disconnect().await;
//                     break;
//                 },
//             };
//         }
//     }
// }

// impl Clock2 {
//     pub fn create(
//         id: Component2Id,
//         bus: &ClockBusRef,
//         frequency_hz: usize,
//         multiplier: f64,
//     ) -> <Clock2 as Component2>::Ref {
//         // TODO: how should this channel be bounded?
//         let (ref_sender, ref_receiver) = async_channel::bounded(1);
//         let mut component = Self {
//             id,
//             bus: bus.connect(id),
//             current_cycle: 0,
//             frequency_hz,
//             metrics: ClockMetrics::new(),
//             multiplier,
//             ref_receiver: ref_receiver.clone(),
//             running: false,
//         };
//         tokio::spawn(async move { component.run().await });
//         <Clock2 as Component2>::Ref::new(&id, ref_receiver, ref_sender)
//     }
// }

// pub type Clock2Ref = Component2Ref<Clock2Message>;

// impl Clock2Ref {
//     pub async fn run_cycles(&self, num_cycles: usize) {
//         self.send(Clock2Message::RunCycles(num_cycles)).await
//     }

//     pub async fn start(&self) {
//         self.send(Clock2Message::StartClock).await
//     }

//     pub async fn stop(&self) {
//         self.send(Clock2Message::StopClock).await
//     }
// }
