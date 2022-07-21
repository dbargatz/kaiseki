use kaiseki_core::{Bus, Component, Machine, Oscillator, RAM, Result, Runner};
use crate::cpu::{Chip8CPU, Chip8RAM};

#[derive(Debug)]
pub struct Chip8Machine {
    bus: Bus,
    cpu: Runner<Chip8CPU>,
    ram: Runner<Chip8RAM>,
    system_clock: Runner<Oscillator>,
}

impl Component for Chip8Machine {
    fn connect_to_bus(&mut self, bus: kaiseki_core::BusConnection) {
        println!("cannot connect machine to bus");
    }

    fn start(&mut self) {
        println!("starting Chip-8 machine");

        self.cpu.start();
        self.ram.start();
        self.system_clock.start();

        self.cpu.stop();
    }
}

impl Machine for Chip8Machine { }

impl Chip8Machine {
    pub fn new(program: &[u8]) -> Result<Chip8Machine> {
        let bus = Bus::new();
        let mut cpu = Chip8CPU::new(0x200);
        let mut ram = Chip8RAM::new();
        let mut osc = Oscillator::new(500);

        bus.connect(&mut ram);
        bus.connect(&mut cpu);
        bus.connect(&mut osc);

        ram.write(0x200, program);

        let cpurun = Runner::new(cpu);
        let ramrun = Runner::new(ram);
        let oscrun = Runner::new(osc);

        let machine = Chip8Machine { bus, cpu: cpurun, ram: ramrun, system_clock: oscrun };
        Ok(machine)

        // TODO: BUS BETWEEN CPU AND MEMORY
        // TODO: BUS BETWEEN OSCILLATOR AND CPU (REMOVE OSCILLATOR CLIENT)
        // TODO: MAKE EVERYTHING A COMPONENT
    }
}