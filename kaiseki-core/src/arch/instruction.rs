use crate::cpu::opcode::OpcodeValue;
use std::fmt;

pub use paste::paste;

pub trait InstructionId {}

pub trait Instruction<V: OpcodeValue>: fmt::Debug {
    type Id: InstructionId;

    fn mnemonic() -> &'static str
    where
        Self: Sized;
    fn width_bits() -> u32
    where
        Self: Sized;
    fn valid_opcodes() -> &'static [V]
    where
        Self: Sized;

    fn id(&self) -> Self::Id;
    fn create(value: V) -> Self
    where
        Self: Sized;
    fn opcode(&self) -> V;
}

pub const fn exclusions_contain(value: u16, exclusions: &[u16]) -> bool {
    let mut i = 0;
    while i < exclusions.len() {
        if exclusions[i] == value {
            return true;
        }
        i += 1;
    }
    false
}

pub const fn gen_opcodes<const N: usize>(start: u16, exclusions: &[u16]) -> [u16; N] {
    let mut opcodes: [u16; N] = [0; N];
    let mut i = 0;
    let mut cur = start;
    while i < N {
        if exclusions_contain(cur, exclusions) {
            cur += 1;
            continue;
        }

        opcodes[i] = cur;
        i += 1;
        cur += 1;
    }
    opcodes
}

#[macro_export]
macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}

#[macro_export]
macro_rules! count_exclusions {
    ($($tts:tt),*) => { <[()]>::len(&[$($crate::replace_expr!($tts ())),*]) };
}

#[macro_export]
macro_rules! instruction {
    (@generate_struct $isa_name:ident, $name:ident, $opcode_type:ty) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name {
            value: $opcode_type,
        }

        $crate::arch::instruction::paste! {
            impl [<$isa_name Instruction>] for $name {}
        }
    };

    (@generate_display_impl $name:ident, $mnemonic:literal) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str($mnemonic)
            }
        }
    };

    (@generate_instruction_impl $isa_name:ident, $name:ident, $mnemonic:literal, $opcode_type:ty) => {
        $crate::arch::instruction::paste! {
            impl $crate::arch::instruction::Instruction<$opcode_type> for $name {
                type Id = [<$isa_name InstructionId>];

                fn mnemonic() -> &'static str {
                    $mnemonic
                }

                fn valid_opcodes() -> &'static [$opcode_type] {
                    &[<$name:snake:upper _VALID_OPCODES>]
                }

                fn width_bits() -> u32 {
                    $opcode_type::BITS
                }

                fn id(&self) -> Self::Id {
                    [<$isa_name InstructionId>]::$name
                }

                fn create(value: $opcode_type) -> Self {
                    Self { value }
                }

                fn opcode(&self) -> $opcode_type {
                    self.value
                }
            }
        }
    };

    (@generate_opcode_const $name:ident, $opcode_type:ty, $rhs:expr) => {
        $crate::arch::instruction::paste! {
            const [<$name:snake:upper _VALID_OPCODES>]: &'static [$opcode_type] = $rhs;
        }
    };

    ($isa_name:ident, $opcode_type:ty, $name:ident, $mnemonic:literal, $opcode:literal) => {
        $crate::instruction!(@generate_opcode_const $name, $opcode_type, &[$opcode]);
        $crate::instruction!(@generate_struct $isa_name, $name, $opcode_type);
        $crate::instruction!(@generate_display_impl $name, $mnemonic);
        $crate::instruction!(@generate_instruction_impl $isa_name, $name, $mnemonic, $opcode_type);
    };
    ($isa_name:ident, $opcode_type:ty, $name:ident, $mnemonic:literal, $x:literal..=$y:literal) => {
        $crate::instruction!(@generate_opcode_const $name, $opcode_type, &$crate::arch::instruction::gen_opcodes::<{ $y - $x + 1 }>($x, &[]));
        $crate::instruction!(@generate_struct $isa_name, $name, $opcode_type);
        $crate::instruction!(@generate_display_impl $name, $mnemonic);
        $crate::instruction!(@generate_instruction_impl $isa_name, $name, $mnemonic, $opcode_type);
    };
    ($isa_name:ident, $opcode_type:ty, $name:ident, $mnemonic:literal, $x:literal..=$y:literal except [ $($exc:literal),* ]) => {
        $crate::instruction!(@generate_opcode_const $name, $opcode_type, &$crate::arch::instruction::gen_opcodes::<{ $y - $x + 1 - $crate::count_exclusions!($($exc),*) }>($x, &[$($exc, )*]));
        $crate::instruction!(@generate_struct $isa_name, $name, $opcode_type);
        $crate::instruction!(@generate_display_impl $name, $mnemonic);
        $crate::instruction!(@generate_instruction_impl $isa_name, $name, $mnemonic, $opcode_type);
    };
}

#[macro_export]
macro_rules! instruction_set {
    (@generate_instruction_trait $isa_name:ident, $opcode_type:ty) => {
        $crate::arch::instruction::paste! {
            pub trait [<$isa_name Instruction>]: $crate::arch::instruction::Instruction<$opcode_type> {
                fn address(&self) -> $opcode_type {
                    self.opcode() & 0x0FFF
                }
            }
        }
    };

    (@generate_id_enum $isa_name:ident, $($name:ident),+) => {
        $crate::arch::instruction::paste! {
            #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub enum [<$isa_name InstructionId>] {
                $($name,)+
            }

            impl $crate::arch::instruction::InstructionId for [<$isa_name InstructionId>] {}
        }
    };

    (@handle_subfield $isa_name:ident, $opcode_type:ty, instructions, { $($name:ident ($mnemonic:literal, $($opcode_fields:tt)+),)* }) => {
        $crate::instruction_set!(@generate_instruction_trait $isa_name, $opcode_type);
        $crate::instruction_set!(@generate_id_enum $isa_name, $($name),+);
        $($crate::instruction!($isa_name, $opcode_type, $name, $mnemonic, $($opcode_fields)*);)*
    };

    ($isa_name:ident: $opcode_type:ty { $($subfield_name:ident: $subfield_content:tt)+ }) => {
        $($crate::instruction_set!(@handle_subfield $isa_name, $opcode_type, $subfield_name, $subfield_content))+;
    };
}
