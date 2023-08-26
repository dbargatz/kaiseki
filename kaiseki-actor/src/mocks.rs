use crate::ActorState;
use futures::channel::oneshot;

#[derive(Debug)]
pub struct Tester {
    sum: usize,
}

impl Tester {
    pub fn new(starting_sum: usize) -> Self {
        Self { sum: starting_sum }
    }

    pub fn add(&mut self, num: usize) -> usize {
        self.sum += num;
        self.sum
    }

    pub fn get_sum(&self) -> usize {
        self.sum
    }
}

// TODO: Autogenerate this from proc macro attribute on impl block
pub enum TestMessage {
    Add {
        num: usize,
        response: oneshot::Sender<usize>,
    },
    GetSum {
        response: oneshot::Sender<usize>,
    },
}

// TODO: Autogenerate this from proc macro attribute on impl block
impl ActorState for Tester {
    type Message = TestMessage;

    fn dispatch(&mut self, message: Self::Message) {
        match message {
            TestMessage::Add { num, response } => {
                response
                    .send(self.add(num))
                    .expect("response sent successfully");
            }
            TestMessage::GetSum { response } => {
                response
                    .send(self.get_sum())
                    .expect("response sent successfully");
            }
        }
    }
}
