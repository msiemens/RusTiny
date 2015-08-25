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