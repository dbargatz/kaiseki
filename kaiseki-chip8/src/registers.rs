#[derive(Debug, Default)]
#[allow(non_snake_case)]
#[allow(unused)]
pub struct Chip8Registers {
    pub V0: u8,
    pub V1: u8,
    pub V2: u8,
    pub V3: u8,
    pub V4: u8,
    pub V5: u8,
    pub V6: u8,
    pub V7: u8,
    pub V8: u8,
    pub V9: u8,
    pub VA: u8,
    pub VB: u8,
    pub VC: u8,
    pub VD: u8,
    pub VE: u8,

    // Flags register - do not use as general-purpose register.
    pub VF: u8,
    pub VI: u16,
    pub PC: u16,
    pub SP: u8,

    /// Delay timer register.
    pub DT: u8,

    /// Sound timer register.
    pub ST: u8,
}

impl Chip8Registers {
    pub fn new() -> Self {
        Chip8Registers {
            ..Default::default()
        }
    }
}
