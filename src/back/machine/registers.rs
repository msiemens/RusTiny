#[derive(Debug)]
pub enum MachineRegister {
    // General purpose registers
    AX,
    BX,
    CX,
    DX,

    // Stack management registers
    SP,
    BP,
}