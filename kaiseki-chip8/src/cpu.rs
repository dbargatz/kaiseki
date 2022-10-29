use std::u8;

use anyhow::Result;
use async_trait::async_trait;

use kaiseki_core::{
    AddressableBus, Component, ComponentId, ExecutableComponent, OscillatorBus,
    OscillatorBusMessage,
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
    memory_bus: AddressableBus,
    regs: Chip8Registers,
    stack: Chip8Stack,
}

impl Component for Chip8CPU {
    fn id(&self) -> &ComponentId {
        &self.id
    }
}

#[async_trait]
impl ExecutableComponent for Chip8CPU {
    async fn start(&mut self) {
        loop {
            let (message, responder) = self.clock_bus.recv(&self.id).await.unwrap();
            if let OscillatorBusMessage::CycleBatchStart {
                start_cycle,
                cycle_budget,
            } = message
            {
                let end_cycle = start_cycle + cycle_budget;
                tracing::info!("executing cycles {} - {}", start_cycle, end_cycle);
                for current_cycle in start_cycle..end_cycle {
                    self.execute_cycle(current_cycle).await.unwrap();
                }
                let response = OscillatorBusMessage::CycleBatchEnd {
                    start_cycle,
                    cycles_spent: cycle_budget,
                };
                responder.unwrap().send(response).unwrap();
            }
        }
    }
}

impl Chip8CPU {
    pub fn new(clock_bus: &OscillatorBus, memory_bus: &AddressableBus, initial_pc: u16) -> Self {
        let id = ComponentId::new("Chip-8 CPU");
        let mut cpu = Chip8CPU {
            id,
            clock_bus: clock_bus.clone(),
            memory_bus: memory_bus.clone(),
            regs: Chip8Registers::new(),
            stack: Chip8Stack::new(),
        };
        cpu.regs.PC = initial_pc;
        cpu
    }

    fn draw_sprite(&self, address: u16, length: u8, x_pos: usize, y_pos: usize) -> bool {
        let sprite = self.memory_bus.read(address.into(), length.into()).unwrap();
        let mut pixel_flipped = false;
        for (sprite_row, sprite_byte) in sprite.iter().enumerate() {
            let pixel_y_idx = (y_pos + sprite_row) * 64; // TODO: fix up how width is stored, make const
            for sprite_col in 0..=7 {
                let pixel_x_idx = x_pos + sprite_col;
                let pixel_byte_idx = (pixel_y_idx + pixel_x_idx) / 8;
                let pixel_bit_idx = (pixel_y_idx + pixel_x_idx) % 8;

                let sprite_bit = (sprite_byte & (0x80 >> sprite_col)) >> (7 - sprite_col);
                let sprite_mask = sprite_bit << (7 - pixel_bit_idx);
                let mut pixel_byte = self.memory_bus.read(0x1000 + pixel_byte_idx, 1).unwrap()[0];
                let prev_value = pixel_byte;
                pixel_byte ^= sprite_mask;
                if prev_value != pixel_byte {
                    pixel_flipped = true;
                }
                self.memory_bus
                    .write(0x1000 + pixel_byte_idx, &[pixel_byte])
                    .unwrap();
            }
        }
        pixel_flipped
    }

    async fn fetch(&self, address: u16) -> Result<u16> {
        let bytes = self.memory_bus.read(address as usize, 2)?;
        let slice: [u8; 2] = bytes[0..2]
            .try_into()
            .expect("couldn't convert Vec<u8> to [u8; 2]");
        Ok(u16::from_be_bytes(slice))
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
                    self.memory_bus.write(0x1000, &[0; 2048]).unwrap();
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
                self.regs.VF =
                    match self.draw_sprite(self.regs.VI, embedded_nybble, vx.into(), vy.into()) {
                        true => 1,
                        false => 0,
                    };
                self.regs.PC += 2;
                desc = format!(
                    "draw {}-byte sprite in memory at 0x{:04X} to (V{}: {}, V{}: {})",
                    embedded_nybble, self.regs.VI, vx_id, vx, vy_id, vy
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
