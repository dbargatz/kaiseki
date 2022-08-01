use std::fmt;

pub struct Chip8Stack {
    stack_pointer: u8,
    slots: [u16; 16],
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
        Chip8Stack {
            stack_pointer: 0,
            slots: [0; 16],
        }
    }

    pub fn pop(&mut self) -> u16 {
        assert!(self.stack_pointer > 0);
        let address = self.slots[self.stack_pointer as usize];
        self.stack_pointer -= 1;
        address
    }

    pub fn push(&mut self, address: u16) {
        assert!(self.stack_pointer < 16);
        self.slots[self.stack_pointer as usize] = address;
        self.stack_pointer += 1;
    }
}
