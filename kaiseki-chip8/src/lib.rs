#![feature(trace_macros)]

pub mod cpu;
pub mod machine;

mod arch;
mod decoder;
mod display;
mod instructions;
mod registers;
mod stack;
