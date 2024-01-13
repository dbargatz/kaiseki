#![allow(clippy::identity_op)]
use kaiseki_core::instruction_set;

instruction_set! {
    Chip8: u16 {
        instructions: {
            ClearScreen("CLS", 0x00E0),                   // 0x00E0
            Return("RET", 0x00EE),                        // 0x00EE
            Jump("JP 0x{0:03X}", 0x1000..=0x1FFF),        // 0x1NNN
            Call("CALL 0x{0:03X}", 0x2000..=0x2FFF),      // 0x2NNN
            SkipIfEqual("SE V{0:X}, 0x{1:02X}", 0x3000..=0x3FFF),     // 0x3NNN
            SkipIfNotEqual("SNE V{0:X}, 0x{1:02X}", 0x4000..=0x4FFF), // 0x4NNN
            SkipIfRegEqual("SE V{0:X}, V{1:X}", 0),       // 0x5XY0
            SetReg("LD V{0:X}, 0x{1:02X}", 0),            // 0x6XNN
            AddReg("ADD V{0:X}, 0x{1:02X}", 0),           // 0x7XNN
            SetRegReg("LD V{0:X}, V{1:X}", 0),            // 0x8XY0
            OrRegReg("OR V{0:X}, V{1:X}", 0),             // 0x8XY1
            AndRegReg("AND V{0:X}, V{1:X}", 0),           // 0x8XY2
            XorRegReg("XOR V{0:X}, V{1:X}", 0),           // 0x8XY3
            AddRegReg("ADD V{0:X}, V{1:X}", 0),           // 0x8XY4
            SubRegReg("SUB V{0:X}, V{1:X}", 0),           // 0x8XY5
            ShiftRightReg("SHR V{0:X}, V{1:X}", 0),       // 0x8XY6
            SubRegRegReverse("SUBN V{0:X}, V{1:X}", 0),   // 0x8XY7
            ShiftLeftReg("SHL V{0:X}, V{1:X}", 0),        // 0x8XYE
            SkipIfRegNotEqual("SNE V{0:X}, V{1:X}", 0),   // 0x9XY0
            SetVI("LD VI, 0x{0:03X}", 0),                 // 0xANNN
            JumpPlusV0("JP V0, 0x{0:03X}", 0),            // 0xBNNN
            Random("RND 0x{0:02X}", 0),                   // 0xCXNN
            Draw("DRW V{0:X}, V{1:X}, 0x{2:02X}", 0),     // 0xDXYN
            SkipIfKeyPressed("SKP V{0:X}", 0),            // 0xEX9E
            SkipIfKeyNotPressed("SKNP V{0:X}", 0),        // 0xEXA1
            GetDelayTimer("LD V{0:X}, DT", 0),            // 0xFX07
            WaitForKey("LD V{0:X}, KEY", 0),              // 0xFX0A
            SetDelayTimer("LD DT, V{0:X}", 0),            // 0xFX15
            SetSoundTimer("LD ST, V{0:X}", 0),            // 0xFX18
            AddRegVI("ADD VI, V{0:X}", 0),                // 0xFX1E
            SetVIDigit("LD VI, DIG[V{0:X}]", 0),          // 0xFX29
            StoreBCD("LD [VI], BCD(V{0:X})", 0),          // 0xFX33
            StoreRegs("LD [VI..VI+{0}], V[0..{0:X}]", 0), // 0xFX55
            LoadRegs("LD V[0..{0:X}], [VI..VI+{0}]", 0),  // 0xFX65
            ExecuteMachineSubroutine("SYS 0x{0:03X}", 0x0000..=0x0FFF except [0x00E0, 0x00EE]), // 0x0NNN except 0x00E0 and 0x00EE
        }
    }
}

pub mod chip8 {
    pub mod registers {
        pub enum RegisterId {
            V0,
            V1,
            V2,
            V3,
            V4,
            V5,
            V6,
            V7,
            V8,
            V9,
            VA,
            VB,
            VC,
            VD,
            VE,
            VF,
        }

        impl RegisterId {
            pub fn get_by_index(index: u8) -> RegisterId {
                match index {
                    0x0 => RegisterId::V0,
                    0x1 => RegisterId::V1,
                    0x2 => RegisterId::V2,
                    0x3 => RegisterId::V3,
                    0x4 => RegisterId::V4,
                    0x5 => RegisterId::V5,
                    0x6 => RegisterId::V6,
                    0x7 => RegisterId::V7,
                    0x8 => RegisterId::V8,
                    0x9 => RegisterId::V9,
                    0xA => RegisterId::VA,
                    0xB => RegisterId::VB,
                    0xC => RegisterId::VC,
                    0xD => RegisterId::VD,
                    0xE => RegisterId::VE,
                    0xF => RegisterId::VF,
                    _ => panic!("Invalid register index: {}", index),
                }
            }
        }
    }

    pub mod instructions {
        use kaiseki_macros::fields;

        fields! {
            Opcode: u16 {
                // becomes pub fn kk(&self) -> u8 { (self.value & 0x00FF) as u8 }
                kk: u8 = $[0..=7],
                // becomes pub fn nnn(&self) -> u16 { (self.value & 0x0FFF) as u16 }
                nnn: u16 = $[0..=11],
                // becomes let x: RegisterId = { ... }
                x: u8 = $[8..=11],
                    // x: RegisterId = |raw: u16| {
                    //     let value: u8 = (raw & 0x0F00) >> 8;
                    //     RegisterId::get_by_index(value)
                    // },
                // becomes let y: RegisterId = { ... }
                y: u8 = $[4..=7],
                    // y: RegisterId = |raw: u16| {
                    //     let value: u8 = (raw & 0x00F0) >> 4;
                    //     RegisterId::get_by_index(value)
                    // },
            },
            FlagsReg: u16 {

            }
        }

        // instructions! {
        //     ClearScreen { "CLS", 0x00E0 },
        //     // ....
        //     Call { "CALL", 0x2[nnn] },
        //     // ...
        //     SkipIfEqual { "SE", 0x3[x][kk] },
        //     // ...
        //     Or { "OR", 0x8[x][y]1 },
        // }
    }
}
