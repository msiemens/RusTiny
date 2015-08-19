//! The RusTiny machine description
//!

mod cconv;
#[macro_use] mod instructions;
mod registers;


pub use self::instructions::{Argument, Instruction, MachineCode};
pub use self::registers::MachineRegister;


pub type Word = u64;