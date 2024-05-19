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
    use kaiseki_core::cpu::opcode::Opcode16;
    use kaiseki_macros::fields;
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
        VF: u8,
        VI: u16,
        PC: u16,
        SP: u8,
        DT: u8,
        ST: u8,
    }

    fields! {
        /// Common fields defined in CHIP-8 opcodes.
        Opcode: Opcode16 {
            /// An embedded 8-bit constant in the lowest byte of the opcode. For example, for
            /// opcode `0x3F28`, `kk = 0x3F28 & 0x00FF = 0x28`.
            kk: u8 { self.get_byte(0) },
            /// An embedded 12-bit physical memory address in the lowest byte and low nybble of the
            /// high byte of the opcode. For example, for opcode `0x3F28`,
            /// `nnn = 0x3F28 & 0x0FFF = 0xF28`.
            nnn: u16 { self.value() & 0x0FFF },
            /// An embedded 4-bit register index in the low nybble of the high byte of the opcode.
            /// For example, for opcode `0x3F28`, `x = (0x3F28 & 0x0F00) >> 8 = 0xF`.
            x: RegisterId { RegisterId::get_by_index(self.get_nybble(2)) },
            /// An embedded 4-bit register index in the high nybble of the low byte of the opcode.
            /// For example, for opcode `0x3F28`, `x = (0x3F28 & 0x00F0) >> 4 = 0x2`.
            y: RegisterId { RegisterId::get_by_index(self.get_nybble(1)) },
        },
    }

    pub mod instructions {

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
