use kaiseki_macros::registers;

registers! {
    V0: u8,
    V1: u8,
    V2: u8,
    V3: u8,
    V4: u8,
    V5: u8,
    V6: u8,
    V7: u8,
    V8: u8,
    V9: u8,
    VA: u8,
    VB: u8,
    VC: u8,
    VD: u8,
    VE: u8,

    // Flags register - do not use as general-purpose register.
    VF: u8,
    VI: u16,
    PC: u16,
    SP: u8,

    // Delay timer register.
    DT: u8,

    // Sound timer register.
    ST: u8,
}

impl RegisterSet {
    pub fn new() -> Self {
        RegisterSet {
            ..Default::default()
        }
    }

    pub fn get_register_ref(&self, index: u8) -> &Register<u8> {
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

    pub fn get_register_mut(&mut self, index: u8) -> &mut Register<u8> {
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
