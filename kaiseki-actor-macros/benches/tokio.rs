use std::time::Duration;

use criterion::Criterion;
use criterion::{criterion_group, criterion_main};

use kaiseki_actor_macros::actor;
use tokio::runtime::Runtime;

#[derive(Debug)]
pub struct Tester {
    sum: usize,
}

#[actor(tokio)]
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

fn tokio_add(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();
    let tester;
    {
        let _guard = runtime.enter();
        tester = TesterActor::create(Tester::new(0));
    }

    c.bench_function("tokio_add", |b| {
        b.to_async(&runtime).iter(|| async {
            let _ = tester.clone().add(1).await;
        });
    });
}

criterion_group!(
    name=benches;
    config=Criterion::default(); // get_config();
    targets=tokio_add
);
criterion_main!(benches);
