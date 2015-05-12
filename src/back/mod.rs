mod instsel;
mod machine;


pub use self::instsel::select_instructions;


// IR -> ASM (v)
//pub fn instruction_selection() {};

// ASM (v) -> ASM (v)
//pub fn instruction_scheduling() {};

// ASM (v) -> ASM
//pub fn register_allocation() {};

// ASM -> ASM
//pub fn prolog_epilog_insertion() {};

//pub fn emit_code(emitter: Box<CodeEmitter>) {};