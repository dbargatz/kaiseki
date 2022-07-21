use std::sync::Mutex;

use bytes::Bytes;

use crate::bus::{BusConnection, BusMessage};
use crate::component::Component;

pub trait RAM: Component {
    fn read(&self, addr: usize, len: usize) -> Bytes;
    fn read_u8(&self, addr: usize) -> u8;
    fn read_u16(&self, addr: usize) -> u16;
    //fn read_u32(&self, addr: usize) -> u32;
    //fn read_u64(&self, addr: usize) -> u64;

    fn write(&self, addr: usize, bytes: &[u8]);
}

#[derive(Debug)]
pub struct SimpleRAM<const N: usize> {
    bus: Option<BusConnection>,
    memory: Mutex<[u8; N]>,
}

impl<const N: usize> Component for SimpleRAM<N> {
    fn connect_to_bus(&mut self, bus: BusConnection) {
        self.bus = Some(bus);
    }

    fn start(&mut self) {
        let bus = self.bus.as_mut().unwrap();
        loop {
            let msg = bus.recv().unwrap();
            match msg {
                BusMessage::ReadAddress { address, length, response_channel } => {
                    let end_addr = address + length;
                    let slice: &[u8] = &self.memory.lock().unwrap()[address..end_addr];
                    let mem = bytes::Bytes::copy_from_slice(slice);
                    response_channel.send(mem);
                },
                _ => { }
            }
        }
    }
}

impl<const N: usize> RAM for SimpleRAM<N> {
    fn read(&self, addr: usize, len: usize) -> Bytes {
        let memory = self.memory.lock().unwrap();
        Bytes::copy_from_slice(&memory[addr..addr+len])
    }

    fn read_u8(&self, addr: usize) -> u8 {
        let slice = self.read(addr, 1);
        slice[0]
    }

    fn read_u16(&self, addr: usize) -> u16 {
        let slice = self.read(addr, 2);
        let value: u16 = (slice[0] as u16) << 8 as u16 | slice[1] as u16;
        value
    }

    fn write(&self, addr: usize, bytes: &[u8]) {
        let mut memory = self.memory.lock().unwrap();
        let mut address = addr;
        for byte in bytes {
            memory[address] = *byte;
            address += 1;
        }
    }
}

impl<const N: usize> SimpleRAM<N> {
    pub fn new() -> Self {
        SimpleRAM { bus: None, memory: Mutex::new([0; N]) }
    }
}
