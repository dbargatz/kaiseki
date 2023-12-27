use std::sync::Arc;
use std::u8;

use async_trait::async_trait;
use thiserror::Error;
use tokio::sync::RwLock;

use kaiseki_core::{
    AddressableBus, AddressableComponent, AddressableComponentError, Component, ComponentId,
    ExecutableComponent, OscillatorBus, OscillatorBusMessage,
};

use super::registers::Chip8Registers;
use super::stack::Chip8Stack;

#[derive(Debug, Error, PartialEq)]
pub enum Chip8CpuError {
    #[error("failed to fetch next instruction")]
    InstructionFetch(#[from] AddressableComponentError),
}

pub type Result<T> = std::result::Result<T, Chip8CpuError>;

#[derive(Debug)]
pub struct Chip8CPU {
    id: ComponentId,
    clock_bus: OscillatorBus,
    memory_bus: AddressableBus,
    regs: Arc<RwLock<Chip8Registers>>,
    stack: Arc<RwLock<Chip8Stack>>,
}

impl Component for Chip8CPU {
    fn id(&self) -> &ComponentId {
        &self.id
    }
}

#[async_trait]
impl ExecutableComponent for Chip8CPU {
    async fn start(&self) {
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
        let mut regs = Chip8Registers::new();
        regs.PC.write(initial_pc);
        Chip8CPU {
            id,
            clock_bus: clock_bus.clone(),
            memory_bus: memory_bus.clone(),
            regs: Arc::new(RwLock::new(regs)),
            stack: Arc::new(RwLock::new(Chip8Stack::new())),
        }
    }

    fn draw_sprite(&self, address: u16, length: u8, x_pos: usize, y_pos: usize) -> bool {
        let sprite = self.memory_bus.read(address.into(), length.into()).unwrap();
        let mut pixel_flipped = false;
        for (sprite_row, sprite_byte) in sprite.iter().enumerate() {
            let display_row_offset = (y_pos + sprite_row) * 8;
            for sprite_col in 0..=7 {
                let sprite_bit = (sprite_byte >> (7 - sprite_col)) & 0x01;
                // Absolute 0 - 63 column index into the display for the current pixel.
                let display_col = (x_pos + sprite_col) % 64;
                // Offset that the column index provides to the final display byte index.
                let display_col_offset = display_col / 8;
                let display_byte_idx = display_row_offset + display_col_offset;
                let display_byte = self.memory_bus.read(0x1000 + display_byte_idx, 1).unwrap()[0];
                let display_bit_idx = 7 - (display_col % 8);
                let display_bitmask = 0x01 << display_bit_idx;
                let display_bit = (display_byte & display_bitmask) >> display_bit_idx;
                let new_bit = display_bit ^ sprite_bit;
                if new_bit != display_bit {
                    pixel_flipped = true;
                }
                let new_byte = (display_byte & !display_bitmask) | (new_bit << display_bit_idx);
                self.memory_bus
                    .write(0x1000 + display_byte_idx, &[new_byte])
                    .unwrap();
            }
        }
        pixel_flipped
    }

    fn fetch(&self, address: u16) -> Result<u16> {
        let bytes = self.memory_bus.read(address as usize, 2)?;
        let slice: [u8; 2] = bytes[0..2]
            .try_into()
            .expect("couldn't convert Vec<u8> to [u8; 2]");
        Ok(u16::from_be_bytes(slice))
    }

    async fn execute_cycle(&self, cycle_number: usize) -> Result<()> {
        let mut regs = self.regs.write().await;

        let opcode = self.fetch(regs.PC.read())?;
        let embedded_address = opcode & 0x0FFF;
        let embedded_byte = (opcode & 0x00FF) as u8;
        let embedded_nybble = (opcode & 0x000F) as u8;
        let vx_id = ((opcode & 0x0F00) >> 8) as u8;
        let vy_id = ((opcode & 0x00F0) >> 4) as u8;
        let desc: String;
        match opcode {
            0x0000..=0x0FFF => match opcode {
                0x00E0 => {
                    regs.PC += 2;
                    self.memory_bus.write(0x1000, &[0; 2048]).unwrap();
                    desc = String::from("clear screen");
                }
                0x00EE => {
                    let mut stack = self.stack.write().await;
                    regs.PC.write(stack.pop());
                    desc = String::from("return from subroutine");
                }
                _ => {
                    regs.PC.write(embedded_address);
                    desc = format!(
                        "execute machine language subroutine at 0x{:04X}",
                        embedded_address,
                    );
                }
            },
            0x1000..=0x1FFF => {
                regs.PC.write(embedded_address);
                desc = format!("jump to address 0x{:04X}", embedded_address);
            }
            0x2000..=0x2FFF => {
                let mut stack = self.stack.write().await;
                stack.push(regs.PC.read() + 2);
                regs.PC.write(embedded_address);
                desc = format!("execute subroutine at address 0x{:04X}", embedded_address);
            }
            0x3000..=0x3FFF => {
                let vx = regs.get_register_ref(vx_id);
                if vx.read() == embedded_byte {
                    regs.PC += 4;
                } else {
                    regs.PC += 2;
                }
                desc = format!(
                    "skip next instruction if V{} == 0x{:02X}",
                    vx_id, embedded_byte
                );
            }
            0x4000..=0x4FFF => {
                let vx = regs.get_register_ref(vx_id);
                if vx.read() != embedded_byte {
                    regs.PC += 4;
                } else {
                    regs.PC += 2;
                }
                desc = format!(
                    "skip next instruction if V{} != 0x{:02X}",
                    vx_id, embedded_byte
                );
            }
            0x5000..=0x5FFF => match embedded_nybble {
                0x0 => {
                    let vx = regs.get_register_ref(vx_id);
                    let vy = regs.get_register_ref(vy_id);
                    if vx.read() == vy.read() {
                        regs.PC += 4;
                    } else {
                        regs.PC += 2;
                    }
                    desc = format!("skip next instruction if V{} == V{}", vx_id, vy_id);
                }
                _ => panic!("invalid 0x5XY0 opcode"),
            },
            0x6000..=0x6FFF => {
                let vx = regs.get_register_mut(vx_id);
                vx.write(embedded_byte);
                regs.PC += 2;
                desc = format!("store 0x{:02X} in V{}", embedded_byte, vx_id);
            }
            0x7000..=0x7FFF => {
                let vx = regs.get_register_mut(vx_id);
                vx.write(vx.read() + embedded_byte);
                regs.PC += 2;
                desc = format!("add 0x{:02X} to V{}", embedded_byte, vx_id);
            }
            0x8000..=0x8FFF => match embedded_nybble {
                0x0 => {
                    let vy_value = regs.get_register_ref(vy_id).read();
                    let vx = regs.get_register_mut(vx_id);
                    vx.write(vy_value);
                    regs.PC += 2;
                    desc = format!("store V{} in V{}", vy_id, vx_id);
                }
                0x1 => {
                    let vy_value = regs.get_register_ref(vy_id).read();
                    let vx = regs.get_register_mut(vx_id);
                    vx.write(vx.read() | vy_value);
                    regs.PC += 2;
                    desc = format!("store V{} | V{} in V{}", vx_id, vy_id, vx_id);
                }
                0x2 => {
                    let vy_value = regs.get_register_ref(vy_id).read();
                    let vx = regs.get_register_mut(vx_id);
                    vx.write(vx.read() & vy_value);
                    regs.PC += 2;
                    desc = format!("store V{} & V{} in V{}", vx_id, vy_id, vx_id);
                }
                0x3 => {
                    let vy_value = regs.get_register_ref(vy_id).read();
                    let vx = regs.get_register_mut(vx_id);
                    vx.write(vx.read() ^ vy_value);
                    regs.PC += 2;
                    desc = format!("store V{} ^ V{} in V{}", vx_id, vy_id, vx_id);
                }
                0x4 => {
                    let vy_value: u16 = regs.get_register_ref(vy_id).try_read_as().unwrap();
                    let vx = regs.get_register_mut(vx_id);
                    let result = vx.read() as u16 + vy_value;
                    vx.write(result as u8);
                    if result > u8::MAX as u16 {
                        regs.VF.write(0x01);
                    } else {
                        regs.VF.write(0x00);
                    }
                    regs.PC += 2;
                    desc = format!("add V{} + V{} with carry", vx_id, vy_id);
                }
                0x5 => {
                    let vy = regs.get_register_ref(vy_id);
                    let vy_value = vy.read();
                    let vx = regs.get_register_mut(vx_id);
                    let vx_value = vx.read();
                    let borrow = vy_value > vx_value;
                    let result = vx_value as u16 - vy_value as u16;
                    vx.write(result as u8);
                    if borrow {
                        regs.VF.write(0x01);
                    } else {
                        regs.VF.write(0x00);
                    }
                    regs.PC += 2;
                    desc = format!("subtract V{} - V{} with borrow", vx_id, vy_id);
                }
                0x6 => {
                    let vy = regs.get_register_ref(vy_id);
                    let vy_value = vy.read();
                    let vx = regs.get_register_mut(vx_id);
                    let lsb = vy_value & 0x01;
                    vx.write(vy_value >> 1);
                    regs.VF.write(lsb);
                    regs.PC += 2;
                    desc = format!("store V{} >> 1 in V{} with lsb in VF", vy_id, vx_id);
                }
                0x7 => {
                    let vy = regs.get_register_ref(vy_id);
                    let vy_value = vy.read();
                    let vx = regs.get_register_mut(vx_id);
                    let vx_value = vx.read();
                    let borrow = vx_value > vy_value;
                    let result = vy_value as u16 - vx_value as u16;
                    vx.write(result as u8);
                    if borrow {
                        regs.VF.write(0x01);
                    } else {
                        regs.VF.write(0x00);
                    }
                    regs.PC += 2;
                    desc = format!("subtract V{} - V{} with borrow", vy_id, vx_id);
                }
                0xE => {
                    let vy = regs.get_register_ref(vy_id);
                    let vy_value = vy.read();
                    let vx = regs.get_register_mut(vx_id);
                    let msb = (vy_value & 0x80) >> 8;
                    vx.write(vy_value << 1);
                    regs.VF.write(msb);
                    regs.PC += 2;
                    desc = format!("store V{} << 1 in V{} with msb in VF", vy_id, vx_id);
                }
                _ => panic!("invalid 0x8XYN opcode"),
            },
            0x9000..=0x9FFF => match embedded_nybble {
                0x0 => {
                    let vx = regs.get_register_ref(vx_id);
                    let vy = regs.get_register_ref(vy_id);
                    if vx.read() != vy.read() {
                        regs.PC += 4;
                    } else {
                        regs.PC += 2;
                    }
                    desc = format!("skip next instruction if V{} != V{}", vx_id, vy_id);
                }
                _ => panic!("invalid 0x9XY0 opcode"),
            },
            0xA000..=0xAFFF => {
                regs.VI.write(embedded_address);
                regs.PC += 2;
                desc = format!("store 0x{:04X} in VI", embedded_address);
            }
            0xB000..=0xBFFF => {
                let v0_value: u16 = regs.V0.try_read_as().unwrap();
                regs.PC.write(embedded_address + v0_value);
                desc = format!("jump to address 0x{:04X} + V0", embedded_address);
            }
            0xC000..=0xCFFF => {
                let vx = regs.get_register_mut(vx_id);
                let random = rand::random::<u8>();
                vx.write(random & embedded_byte);
                regs.PC += 2;
                desc = format!(
                    "set V{} to a random number with mask 0x{:02X}",
                    vx_id, embedded_byte
                );
            }
            0xD000..=0xDFFF => {
                let vx_value: usize = regs.get_register_ref(vx_id).try_read_as().unwrap();
                let vy_value: usize = regs.get_register_ref(vy_id).try_read_as().unwrap();
                if self.draw_sprite(regs.VI.read(), embedded_nybble, vx_value, vy_value) {
                    regs.VF.write(0x01);
                } else {
                    regs.VF.write(0x00);
                };
                regs.PC += 2;
                desc = format!(
                    "draw {}-byte sprite in memory at {} to (V{}: 0x{:04X}, V{}: 0x{:04X})",
                    embedded_nybble, regs.VI, vx_id, vx_value, vy_id, vy_value
                );
            }
            _ => panic!("invalid opcode: 0x{:04X}", opcode),
        }

        tracing::debug!(
            "cycle {} | PC: {} | 0x{:04X} {}",
            cycle_number,
            regs.PC,
            opcode,
            desc,
        );

        Ok(())
    }
}
