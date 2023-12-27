use kaiseki_core::register::{Reg16, Reg8};

#[derive(Debug, Default)]
#[allow(non_snake_case)]
#[allow(unused)]
pub struct Chip8Registers {
    pub V0: Reg8,
    pub V1: Reg8,
    pub V2: Reg8,
    pub V3: Reg8,
    pub V4: Reg8,
    pub V5: Reg8,
    pub V6: Reg8,
    pub V7: Reg8,
    pub V8: Reg8,
    pub V9: Reg8,
    pub VA: Reg8,
    pub VB: Reg8,
    pub VC: Reg8,
    pub VD: Reg8,
    pub VE: Reg8,

    // Flags register - do not use as general-purpose register.
    pub VF: Reg8,
    pub VI: Reg16,
    pub PC: Reg16,
    pub SP: Reg8,

    /// Delay timer register.
    pub DT: Reg8,

    /// Sound timer register.
    pub ST: Reg8,
}

impl Chip8Registers {
    pub fn new() -> Self {
        Chip8Registers {
            ..Default::default()
        }
    }

    pub fn get_register_ref(&self, index: u8) -> &Reg8 {
        match index {
            0x0 => &self.V0,
            0x1 => &self.V1,
            0x2 => &self.V2,
            0x3 => &self.V3,
            0x4 => &self.V4,
            0x5 => &self.V5,
            0x6 => &self.V6,
            0x7 => &self.V7,
            0x8 => &self.V8,
            0x9 => &self.V9,
            0xA => &self.VA,
            0xB => &self.VB,
            0xC => &self.VC,
            0xD => &self.VD,
            0xE => &self.VE,
            0xF => &self.VF,
            _ => panic!("invalid register index 0x{:02X}", index),
        }
    }

    pub fn get_register_mut(&mut self, index: u8) -> &mut Reg8 {
        match index {
            0x0 => &mut self.V0,
            0x1 => &mut self.V1,
            0x2 => &mut self.V2,
            0x3 => &mut self.V3,
            0x4 => &mut self.V4,
            0x5 => &mut self.V5,
            0x6 => &mut self.V6,
            0x7 => &mut self.V7,
            0x8 => &mut self.V8,
            0x9 => &mut self.V9,
            0xA => &mut self.VA,
            0xB => &mut self.VB,
            0xC => &mut self.VC,
            0xD => &mut self.VD,
            0xE => &mut self.VE,
            0xF => &mut self.VF,
            _ => panic!("invalid register index 0x{:02X}", index),
        }
    }
}
