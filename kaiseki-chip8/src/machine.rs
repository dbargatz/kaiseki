use crate::cpu::{Chip8CPU, Chip8RAM};
use kaiseki_core::{
    Component, ComponentId, Machine, MemoryBus, Oscillator, OscillatorBus, Result, Runner, RAM,
};

#[derive(Debug)]
pub struct Chip8Machine {
    id: ComponentId,
    clock_bus: Runner<OscillatorBus>,
    memory_bus: Runner<MemoryBus>,
    cpu: Runner<Chip8CPU>,
    ram: Runner<Chip8RAM>,
    system_clock: Runner<Oscillator>,
}

impl Component for Chip8Machine {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn start(&self) {
        tracing::info!("starting Chip-8 machine");

        self.clock_bus.start();
        self.memory_bus.start();

        self.cpu.start();
        self.ram.start();
        self.system_clock.start();

        self.cpu.stop();
    }
}

impl Machine for Chip8Machine {}

impl Chip8Machine {
    pub fn new(program: &[u8]) -> Result<Chip8Machine> {
        let mut clock_bus = OscillatorBus::new();
        let mut memory_bus = MemoryBus::new();

        let cpu = Chip8CPU::new(&mut clock_bus, &mut memory_bus, 0x200);
        let ram = Chip8RAM::new(&mut memory_bus);
        let osc = Oscillator::new(&mut clock_bus, 5000);

        ram.write(0x200, program);

        let cpurun = Runner::new(cpu);
        let ramrun = Runner::new(ram);
        let oscrun = Runner::new(osc);
        let clock_busrun = Runner::new(clock_bus);
        let memory_busrun = Runner::new(memory_bus);

        let machine = Chip8Machine {
            id: ComponentId::new_v4(),
            clock_bus: clock_busrun,
            memory_bus: memory_busrun,
            cpu: cpurun,
            ram: ramrun,
            system_clock: oscrun,
        };
        Ok(machine)

        // TODO: BUS BETWEEN CPU AND MEMORY
        // TODO: BUS BETWEEN OSCILLATOR AND CPU (REMOVE OSCILLATOR CLIENT)
        // TODO: MAKE EVERYTHING A COMPONENT
    }
}
