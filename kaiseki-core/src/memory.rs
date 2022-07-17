use bytes::Bytes;
use std::sync::Mutex;
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
    memory: Mutex<[u8; N]>,
}

impl<const N: usize> Component for SimpleRAM<N> {}

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
        SimpleRAM { memory: Mutex::new([0; N]) }
    }
}
