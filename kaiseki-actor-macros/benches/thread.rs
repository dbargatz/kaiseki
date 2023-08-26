use std::time::Duration;

use criterion::Criterion;
use criterion::{criterion_group, criterion_main};

use kaiseki_actor_macros::actor;

#[derive(Debug)]
pub struct Tester {
    sum: usize,
}

#[actor(thread)]
impl Tester {
    #[allow(dead_code)]
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

fn get_config() -> Criterion {
    Criterion::default()
        .sample_size(10000)
        .warm_up_time(Duration::from_secs(10))
        .measurement_time(Duration::from_secs(20))
}

fn thread_add(c: &mut Criterion) {
    let mut tester = TesterActor::create(Tester::new(0));
    c.bench_function("thread_add", |b| {
        b.iter(|| tester.add(1));
    });
}

criterion_group!(
    name=benches;
    config=Criterion::default(); // get_config();
    targets=thread_add
);
criterion_main!(benches);
