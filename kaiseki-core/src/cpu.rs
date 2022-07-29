use std::sync::Arc;
use std::thread::JoinHandle;

use async_trait::async_trait;
use tokio::sync::{mpsc, Mutex};

use crate::component::{Component, ComponentId};
use crate::{BusConnection, OscillatorBus, OscillatorBusMessage};

pub struct CpuError;
pub type CpuResult<T> = std::result::Result<T, CpuError>;

pub trait CpuComponent: Component {
    fn execute_cycles(
        &mut self,
        start_cycle: usize,
        end_cycle: usize,
    ) -> CpuResult<()>;
}

#[derive(Debug)]
pub struct Cpu<T: CpuComponent> {
    inner: Arc<Mutex<T>>,
    clock_bus: BusConnection<OscillatorBusMessage>,
    exec_thread: Option<JoinHandle<()>>,
}

#[async_trait]
impl<T: CpuComponent> Component for Cpu<T> {
    fn id(&self) -> ComponentId {
        self.inner.blocking_lock().id()
    }

    async fn start(&mut self) {
        let (start_tx, mut start_rx) = mpsc::channel(100);
        let (end_tx, mut end_rx) = mpsc::channel(100);
        let cpu = self.inner.clone();
        let handle = std::thread::spawn(move || {
            // let rt = tokio::runtime::Builder::new_current_thread()
            //     .enable_all()
            //     .build()
            //     .unwrap();

            loop {
                let cycles_executed: usize;
                //let (start_cycle, end_cycle) = rt.block_on(start_rx.recv()).unwrap();
                let (start_cycle, end_cycle) = start_rx.blocking_recv().unwrap();

                {
                    //let mut cpu_guard = rt.block_on(cpu.lock());
                    let mut cpu_guard = cpu.blocking_lock();
                    match cpu_guard.execute_cycles(start_cycle, end_cycle) {
                        Ok(_) => cycles_executed = end_cycle - start_cycle,
                        Err(_) => break,
                    }
                }

                //rt.block_on(end_tx.send(cycles_executed)).unwrap();
                end_tx.blocking_send(cycles_executed).unwrap();
            }
        });
        self.exec_thread = Some(handle);

        loop {
            let cycle_msg = self.clock_bus.recv().await.unwrap();
            if let OscillatorBusMessage::CycleBatchStart {
                start_cycle,
                cycle_budget,
            } = cycle_msg
            {
                let end_cycle = start_cycle + cycle_budget;
                tracing::info!("executing cycles {} - {}", start_cycle, end_cycle);
                start_tx.send((start_cycle, end_cycle)).await.unwrap();
                let cycles_executed = end_rx.recv().await.unwrap();

                let cycle_end = OscillatorBusMessage::CycleBatchEnd {
                    start_cycle,
                    cycles_spent: cycles_executed,
                };
                self.clock_bus.send(cycle_end).await.unwrap();
            }
        }
    }
}

impl<T: CpuComponent> Cpu<T> {
    pub fn new(clock_bus: &mut OscillatorBus, cpu: T) -> Self {
        let id = cpu.id();
        let clock_conn = clock_bus.connect(&id);
        Cpu {
            inner: Arc::new(Mutex::new(cpu)),
            clock_bus: clock_conn,
            exec_thread: None,
        }
    }
}
