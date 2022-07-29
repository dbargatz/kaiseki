use std::fmt;

pub struct Chip8Stack {
    pub slots: [u16; 16],
}

impl fmt::Debug for Chip8Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Chip-8 Stack").finish()
    }
}

impl Default for Chip8Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl Chip8Stack {
    pub fn new() -> Self {
        Chip8Stack { slots: [0; 16] }
    }
}
