//! The `RusTiny` machine description
//!

use std::fmt;


pub mod cconv;
#[macro_use] pub mod asm;


pub type Word = u64;


#[derive(Copy, Clone, Debug)]
pub enum MachineRegister {
    // General purpose registers
    RAX,
    RBX,
    RCX,
    RDX,
    RSI,
    RDI,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,

    // Stack management registers
    RSP,
    RBP,

    // Needed for artithmetic left shift
    CL,
}


impl fmt::Display for MachineRegister {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MachineRegister::RAX => write!(f, "rax"),
            MachineRegister::RBX => write!(f, "rbx"),
            MachineRegister::RCX => write!(f, "rcx"),
            MachineRegister::RDX => write!(f, "rdx"),
            MachineRegister::RSI => write!(f, "rsi"),
            MachineRegister::RDI => write!(f, "rdi"),
            MachineRegister::R8 => write!(f, "r8"),
            MachineRegister::R9 => write!(f, "r9"),
            MachineRegister::R10 => write!(f, "r10"),
            MachineRegister::R11 => write!(f, "r11"),
            MachineRegister::R12 => write!(f, "r12"),
            MachineRegister::R13 => write!(f, "r13"),
            MachineRegister::R14 => write!(f, "r14"),
            MachineRegister::R15 => write!(f, "r15"),
            MachineRegister::RSP => write!(f, "rsp"),
            MachineRegister::RBP => write!(f, "rbp"),
            MachineRegister::CL => write!(f, "cl"),
        }
    }
}