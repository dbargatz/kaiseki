use std::u8;

use anyhow::Result;
use async_trait::async_trait;
use bytes::Buf;

use kaiseki_core::{
    Component, ComponentId, DisplayBus, ExecutableComponent, MemoryBus, OscillatorBus,
};

use super::registers::Chip8Registers;
use super::stack::Chip8Stack;

#[derive(Clone, Debug)]
pub enum Chip8CpuError {
    LoadError,
}

#[derive(Debug)]
pub struct Chip8CPU {
    id: ComponentId,
    clock_bus: OscillatorBus,
    display_bus: DisplayBus,
    memory_bus: MemoryBus,
    regs: Chip8Registers,
    stack: Chip8Stack,
}

impl Component for Chip8CPU {
    fn id(&self) -> ComponentId {
        self.id.clone()
    }
}

#[async_trait]
impl ExecutableComponent for Chip8CPU {
    async fn start(&mut self) {
        loop {
            let (start_cycle, cycle_budget) = self.clock_bus.wait(&self.id).await.unwrap();
            let end_cycle = start_cycle + cycle_budget;
            tracing::info!("executing cycles {} - {}", start_cycle, end_cycle);
            for current_cycle in start_cycle..end_cycle {
                self.execute_cycle(current_cycle).await.unwrap();
            }

            self.clock_bus
                .complete(&self.id, start_cycle, cycle_budget)
                .await
                .unwrap();
        }
    }
}

impl Chip8CPU {
    pub fn new(
        clock_bus: &OscillatorBus,
        display_bus: &DisplayBus,
        memory_bus: &MemoryBus,
        initial_pc: u16,
    ) -> Self {
        let id = ComponentId::new("Chip-8 CPU");
        let mut cpu = Chip8CPU {
            id,
            clock_bus: clock_bus.clone(),
            display_bus: display_bus.clone(),
            memory_bus: memory_bus.clone(),
            regs: Chip8Registers::new(),
            stack: Chip8Stack::new(),
        };
        cpu.regs.PC = initial_pc;
        cpu
    }

    async fn fetch(&self, address: u16) -> Result<u16> {
        let mut bytes = self.memory_bus.read(&self.id, address as usize, 2).await?;
        Ok(bytes.get_u16())
    }

    async fn execute_cycle(&mut self, cycle_number: usize) -> Result<()> {
        let opcode = self.fetch(self.regs.PC).await?;
        let embedded_address = opcode & 0x0FFF;
        let embedded_byte = (opcode & 0x00FF) as u8;
        let embedded_nybble = (opcode & 0x000F) as u8;
        let vx_id = ((opcode & 0x0F00) >> 8) as u8;
        let vy_id = ((opcode & 0x00F0) >> 4) as u8;
        let desc: String;
        match opcode {
            0x0000..=0x0FFF => match opcode {
                0x00E0 => {
                    self.regs.PC += 2;
                    self.display_bus.clear(&self.id).await?;
                    desc = String::from("clear screen");
                }
                0x00EE => {
                    self.regs.PC = self.stack.pop();
                    desc = String::from("return from subroutine");
                }
                _ => {
                    self.regs.PC = embedded_address;
                    desc = format!(
                        "execute machine language subroutine at 0x{:04X}",
                        embedded_address
                    );
                }
            },
            0x1000..=0x1FFF => {
                self.regs.PC = embedded_address;
                desc = format!("jump to address 0x{:04X}", embedded_address);
            }
            0x2000..=0x2FFF => {
                self.stack.push(self.regs.PC + 2);
                self.regs.PC = embedded_address;
                desc = format!("execute subroutine at address 0x{:04X}", embedded_address);
            }
            0x3000..=0x3FFF => {
                let vx = self.regs.get_register_ref(vx_id);
                if *vx == embedded_byte {
                    self.regs.PC += 4;
                } else {
                    self.regs.PC += 2;
                }
                desc = format!(
                    "skip next instruction if V{} == 0x{:02X}",
                    vx_id, embedded_byte
                );
            }
            0x4000..=0x4FFF => {
                let vx = self.regs.get_register_ref(vx_id);
                if *vx != embedded_byte {
                    self.regs.PC += 4;
                } else {
                    self.regs.PC += 2;
                }
                desc = format!(
                    "skip next instruction if V{} != 0x{:02X}",
                    vx_id, embedded_byte
                );
            }
            0x5000..=0x5FFF => match embedded_nybble {
                0x0 => {
                    let vx = self.regs.get_register_ref(vx_id);
                    let vy = self.regs.get_register_ref(vy_id);
                    if *vx == *vy {
                        self.regs.PC += 4;
                    } else {
                        self.regs.PC += 2;
                    }
                    desc = format!("skip next instruction if V{} == V{}", vx_id, vy_id);
                }
                _ => panic!("invalid 0x5XY0 opcode"),
            },
            0x6000..=0x6FFF => {
                let vx = self.regs.get_register_mut(vx_id);
                *vx = embedded_byte;
                self.regs.PC += 2;
                desc = format!("store 0x{:02X} in V{}", embedded_byte, vx_id);
            }
            0x7000..=0x7FFF => {
                let vx = self.regs.get_register_mut(vx_id);
                *vx += embedded_byte;
                self.regs.PC += 2;
                desc = format!("add 0x{:02X} to V{}", embedded_byte, vx_id);
            }
            0x8000..=0x8FFF => match embedded_nybble {
                0x0 => {
                    let vy = self.regs.get_register_ref(vy_id);
                    let vy_value = *vy;
                    let vx = self.regs.get_register_mut(vx_id);
                    *vx = vy_value;
                    self.regs.PC += 2;
                    desc = format!("store V{} in V{}", vy_id, vx_id);
                }
                0x1 => {
                    let vy = self.regs.get_register_ref(vy_id);
                    let vy_value = *vy;
                    let vx = self.regs.get_register_mut(vx_id);
                    *vx |= vy_value;
                    self.regs.PC += 2;
                    desc = format!("store V{} | V{} in V{}", vx_id, vy_id, vx_id);
                }
                0x2 => {
                    let vy = self.regs.get_register_ref(vy_id);
                    let vy_value = *vy;
                    let vx = self.regs.get_register_mut(vx_id);
                    *vx &= vy_value;
                    self.regs.PC += 2;
                    desc = format!("store V{} & V{} in V{}", vx_id, vy_id, vx_id);
                }
                0x3 => {
                    let vy = self.regs.get_register_ref(vy_id);
                    let vy_value = *vy;
                    let vx = self.regs.get_register_mut(vx_id);
                    *vx ^= vy_value;
                    self.regs.PC += 2;
                    desc = format!("store V{} ^ V{} in V{}", vx_id, vy_id, vx_id);
                }
                0x4 => {
                    let vy = self.regs.get_register_ref(vy_id);
                    let vy_value = *vy;
                    let vx = self.regs.get_register_mut(vx_id);
                    let result = *vx as u16 + vy_value as u16;
                    *vx = result as u8;
                    if result > u8::MAX as u16 {
                        self.regs.VF = 0x01;
                    } else {
                        self.regs.VF = 0x00;
                    }
                    self.regs.PC += 2;
                    desc = format!("add V{} + V{} with carry", vx_id, vy_id);
                }
                0x5 => {
                    let vy = self.regs.get_register_ref(vy_id);
                    let vy_value = *vy;
                    let vx = self.regs.get_register_mut(vx_id);
                    let borrow = vy_value > *vx;
                    let result = *vx as u16 - vy_value as u16;
                    *vx = result as u8;
                    if borrow {
                        self.regs.VF = 0x01;
                    } else {
                        self.regs.VF = 0x00;
                    }
                    self.regs.PC += 2;
                    desc = format!("subtract V{} - V{} with borrow", vx_id, vy_id);
                }
                0x6 => {
                    let vy = self.regs.get_register_ref(vy_id);
                    let vy_value = *vy;
                    let vx = self.regs.get_register_mut(vx_id);
                    let lsb = vy_value & 0x01;
                    *vx = vy_value >> 1;
                    self.regs.VF = lsb;
                    self.regs.PC += 2;
                    desc = format!("store V{} >> 1 in V{} with lsb in VF", vy_id, vx_id);
                }
                0x7 => {
                    let vy = self.regs.get_register_ref(vy_id);
                    let vy_value = *vy;
                    let vx = self.regs.get_register_mut(vx_id);
                    let borrow = *vx > vy_value;
                    let result = vy_value as u16 - *vx as u16;
                    *vx = result as u8;
                    if borrow {
                        self.regs.VF = 0x01;
                    } else {
                        self.regs.VF = 0x00;
                    }
                    self.regs.PC += 2;
                    desc = format!("subtract V{} - V{} with borrow", vy_id, vx_id);
                }
                0xE => {
                    let vy = self.regs.get_register_ref(vy_id);
                    let vy_value = *vy;
                    let vx = self.regs.get_register_mut(vx_id);
                    let msb = (vy_value & 0x80) >> 8;
                    *vx = vy_value << 1;
                    self.regs.VF = msb;
                    self.regs.PC += 2;
                    desc = format!("store V{} << 1 in V{} with msb in VF", vy_id, vx_id);
                }
                _ => panic!("invalid 0x8XYN opcode"),
            },
            0x9000..=0x9FFF => match embedded_nybble {
                0x0 => {
                    let vx = self.regs.get_register_ref(vx_id);
                    let vy = self.regs.get_register_ref(vy_id);
                    if *vx != *vy {
                        self.regs.PC += 4;
                    } else {
                        self.regs.PC += 2;
                    }
                    desc = format!("skip next instruction if V{} != V{}", vx_id, vy_id);
                }
                _ => panic!("invalid 0x9XY0 opcode"),
            },
            0xA000..=0xAFFF => {
                self.regs.VI = embedded_address;
                self.regs.PC += 2;
                desc = format!("store 0x{:04X} in VI", embedded_address);
            }
            0xB000..=0xBFFF => {
                self.regs.PC = embedded_address + self.regs.V0 as u16;
                desc = format!("jump to address 0x{:04X} + V0", embedded_address);
            }
            0xC000..=0xCFFF => {
                let vx = self.regs.get_register_mut(vx_id);
                let random = rand::random::<u8>();
                *vx = random & embedded_byte;
                self.regs.PC += 2;
                desc = format!(
                    "set V{} to a random number with mask 0x{:02X}",
                    vx_id, embedded_byte
                );
            }
            0xD000..=0xDFFF => {
                let vx = *self.regs.get_register_ref(vx_id);
                let vy = *self.regs.get_register_ref(vy_id);
                self.regs.VF = self
                    .display_bus
                    .draw_sprite(&self.id, self.regs.VI, embedded_nybble, vx, vy)
                    .await?;
                self.regs.PC += 2;
                desc = format!(
                    "draw {}-byte sprite in memory at 0x{:04X} to (V{}, V{})",
                    embedded_nybble, self.regs.VI, vx_id, vy_id
                );
            }
            _ => panic!("invalid opcode: 0x{:04X}", opcode),
        }

        tracing::debug!(
            "cycle {} | PC: 0x{:04X} | 0x{:04X} {}",
            cycle_number,
            self.regs.PC,
            opcode,
            desc,
        );

        Ok(())
    }
}
