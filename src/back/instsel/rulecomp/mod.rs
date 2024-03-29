//! The Instruction Selection Rule Compiler
//!
//! We need a lot of rules how to select assembler instructions based on our
//! IR. The rules are described in `rules.ins.rs` and compiled to `rules.rs`.
//! The compiler used for this task is described in this module. While the
//! lexer and parser are quite straightforward, this file is a real mess.
//! It's full of ugly string formatting and repeating code.
//! Nonetheless, it seems to work.
//!
//! The basic idea of the instruction selector is based on *Introduction to
//! Compiler Design*, chapter 7 which proposes a greedy algorithm. Basically,
//! We compare the IR code with a list of possible matches where the more
//! complex patterns are listed before simpler patterns.
//!
//! TODO: Liveness analysis for registers
//!

mod ast;
mod lexer;
mod parser;
mod tokens;

use self::ast::*;
use driver::interner::Ident;
use front::ast::Node;
use std::collections::HashMap;
use std::fmt;

pub fn setup() {
    // Load all keywords into the interning table
    tokens::Keyword::setup();
}

pub fn compile_rules(input: &str, filename: &str) -> String {
    setup();

    let mut parser = parser::Parser::new(lexer::Lexer::new(input, filename));
    let rules = parser.parse();

    format!(
        "// AUTOGENERATED CODE! DO NOT EDIT!
use driver::interner::Ident;
use middle::ir;
use back::machine::{{self, MachineRegister}};
use back::machine::cconv;
use back::machine::asm;

#[derive(Debug)]
enum IrLine<'a> {{
    Instruction(&'a ir::Instruction),
    CFInstruction(&'a ir::ControlFlowInstruction),
}}

#[allow(non_shorthand_field_patterns)]
#[allow(unused_variables)]
pub fn trans_instr(instr: &[&ir::Instruction],
                   last: &ir::ControlFlowInstruction,
                   code: &mut asm::Block)
                   -> (usize, bool)
{{
    let mut lines: Vec<_> = instr.iter().map(|i| IrLine::Instruction(i)).collect();
    lines.push(IrLine::CFInstruction(last));

    match *lines {{
{},
        _ => {{
            println!(\"instr: {{:?}}\", instr);
            println!(\"last: {{:?}}\", last);
            println!(\"lines: {{:?}}\", lines);
            panic!(\"No rule to translate {{:?}} to asm\", lines)
        }}
    }}
}}
",
        rules
            .iter()
            .map(|r| translate_rule(r))
            .collect::<Vec<_>>()
            .join(",\n")
    )
}

fn translate_rule(rule: &Rule) -> String {
    let mut arg_types = HashMap::new();
    for pattern in &rule.pattern.ir_patterns {
        update_pattern_types(pattern, &mut arg_types);
    }
    if let Some(l) = rule.pattern.last.as_ref() {
        update_pattern_last_types(l, &mut arg_types);
    }

    let mut s = "        ".to_owned();
    s.push_str(&translate_pattern(&rule.pattern));
    s.push_str(" => {\n");

    match rule.implementation {
        Impl::Asm(ref asm) => {
            asm.iter()
                .inspect(|instr| {
                    instr
                        .args
                        .iter()
                        .inspect(|arg| {
                            if let AsmArg::NewRegister(name) = ***arg {
                                arg_types.insert(
                                    *name,
                                    IrArg::Register(IrRegister(*name, IrRegisterKind::Local)),
                                );
                            }
                        })
                        .all(|_| true);
                })
                .all(|_| true);

            s.push_str(&translate_asm(asm, &arg_types));
        }
        Impl::Rust(source) => {
            s.push_str(&source);
        }
    };

    s.push_str(&format!(
        "            ({}, {:?})\n",
        rule.pattern.ir_patterns.len(),
        rule.pattern.last.is_some()
    ));
    s.push_str("        }");

    s
}

fn update_pattern_types(pattern: &IrPattern, map: &mut HashMap<Ident, IrArg>) {
    macro_rules! binop {
        ($map:ident, $dest:ident, $lhs:ident, $rhs:ident) => {{
            $map.insert($dest.0, IrArg::Register(IrRegister($dest.0, $dest.1)));
            $map.insert($lhs.get_name(), (**$lhs).clone());
            $map.insert($rhs.get_name(), (**$rhs).clone());
        }};
    }

    match *pattern {
        IrPattern::Add(ref dest, ref lhs, ref rhs)
        | IrPattern::Sub(ref dest, ref lhs, ref rhs)
        | IrPattern::Mul(ref dest, ref lhs, ref rhs)
        | IrPattern::Div(ref dest, ref lhs, ref rhs)
        | IrPattern::Pow(ref dest, ref lhs, ref rhs)
        | IrPattern::Mod(ref dest, ref lhs, ref rhs)
        | IrPattern::Shl(ref dest, ref lhs, ref rhs)
        | IrPattern::Shr(ref dest, ref lhs, ref rhs)
        | IrPattern::And(ref dest, ref lhs, ref rhs)
        | IrPattern::Or(ref dest, ref lhs, ref rhs)
        | IrPattern::Xor(ref dest, ref lhs, ref rhs)
        | IrPattern::CmpLt(ref dest, ref lhs, ref rhs)
        | IrPattern::CmpLe(ref dest, ref lhs, ref rhs)
        | IrPattern::CmpEq(ref dest, ref lhs, ref rhs)
        | IrPattern::CmpNe(ref dest, ref lhs, ref rhs)
        | IrPattern::CmpGe(ref dest, ref lhs, ref rhs)
        | IrPattern::CmpGt(ref dest, ref lhs, ref rhs) => binop!(map, dest, lhs, rhs),
        IrPattern::Neg(ref dest, ref arg) | IrPattern::Not(ref dest, ref arg) => {
            map.insert(dest.0, IrArg::Register(IrRegister(dest.0, dest.1)));
            map.insert(arg.get_name(), (**arg).clone());
        }
        IrPattern::Alloca(ref dest) | IrPattern::Call(ref dest, _, _) => {
            map.insert(dest.0, IrArg::Register(IrRegister(dest.0, dest.1)));
        }
        IrPattern::Load(ref dest, ref addr) => {
            map.insert(dest.0, IrArg::Register(IrRegister(dest.0, dest.1)));
            map.insert(addr.get_name(), (**addr).clone());
        }
        IrPattern::Store(ref value, ref addr) => {
            map.insert(value.get_name(), (**value).clone());
            map.insert(addr.get_name(), (**addr).clone());
        }
    }
}

fn update_pattern_last_types(pattern: &IrPatternLast, map: &mut HashMap<Ident, IrArg>) {
    match *pattern {
        IrPatternLast::Ret(Some(ref val)) => {
            map.insert(val.get_name(), (**val).clone());
        }
        IrPatternLast::Br(ref cond, _, _) => {
            map.insert(cond.get_name(), (**cond).clone());
        }
        IrPatternLast::Ret(None) | IrPatternLast::Jmp(_) => {}
    }
}

fn translate_pattern(pattern: &Pattern) -> String {
    let mut s = "[".to_owned();
    let patterns: Vec<_> = pattern
        .ir_patterns
        .iter()
        .map(|p| translate_ir_pattern(p))
        .collect();
    s.push_str(&patterns.join(", "));

    match pattern.last {
        Some(ref last) => {
            // FIXME: Proper solution (unify instr and last?)
            if !patterns.is_empty() {
                s.push_str(", ");
            }

            s.push_str(&translate_ir_pattern_last(last));
        }
        None => s.push_str(", .."),
    }

    s.push_str("]");

    if let Some(snippet) = pattern.cond {
        s.push_str(&format!(" if {}", snippet));
    }

    s
}

fn translate_ir_pattern(ir_pattern: &IrPattern) -> String {
    match *ir_pattern {
        IrPattern::Add(ref dest, ref arg1, ref arg2) => {
            format!("IrLine::Instruction(&ir::Instruction::BinOp {{ op: ir::InfixOp::Add, lhs: {}, rhs: {}, dst: {} }})",
                    translate_ir_arg(arg1),
                    translate_ir_arg(arg2),
                    translate_ir_register(dest))
        }
        IrPattern::Sub(ref dest, ref arg1, ref arg2) => {
            format!("IrLine::Instruction(&ir::Instruction::BinOp {{ op: ir::InfixOp::Sub, lhs: {}, rhs: {}, dst: {} }})",
                    translate_ir_arg(arg1),
                    translate_ir_arg(arg2),
                    translate_ir_register(dest))
        }
        IrPattern::Mul(ref dest, ref arg1, ref arg2) => {
            format!("IrLine::Instruction(&ir::Instruction::BinOp {{ op: ir::InfixOp::Mul, lhs: {}, rhs: {}, dst: {} }})",
                    translate_ir_arg(arg1),
                    translate_ir_arg(arg2),
                    translate_ir_register(dest))
        }
        IrPattern::Div(ref dest, ref arg1, ref arg2) => {
            format!("IrLine::Instruction(&ir::Instruction::BinOp {{ op: ir::InfixOp::Div, lhs: {}, rhs: {}, dst: {} }})",
                    translate_ir_arg(arg1),
                    translate_ir_arg(arg2),
                    translate_ir_register(dest))
        }
        IrPattern::Pow(ref dest, ref arg1, ref arg2) => {
            format!("IrLine::Instruction(&ir::Instruction::BinOp {{ op: ir::InfixOp::Pow, lhs: {}, rhs: {}, dst: {} }})",
                    translate_ir_arg(arg1),
                    translate_ir_arg(arg2),
                    translate_ir_register(dest))
        }
        IrPattern::Mod(ref dest, ref arg1, ref arg2) => {
            format!("IrLine::Instruction(&ir::Instruction::BinOp {{ op: ir::InfixOp::Mod, lhs: {}, rhs: {}, dst: {} }})",
                    translate_ir_arg(arg1),
                    translate_ir_arg(arg2),
                    translate_ir_register(dest))
        }
        IrPattern::Shl(ref dest, ref arg1, ref arg2) => {
            format!("IrLine::Instruction(&ir::Instruction::BinOp {{ op: ir::InfixOp::Shl, lhs: {}, rhs: {}, dst: {} }})",
                    translate_ir_arg(arg1),
                    translate_ir_arg(arg2),
                    translate_ir_register(dest))
        }
        IrPattern::Shr(ref dest, ref arg1, ref arg2) => {
            format!("IrLine::Instruction(&ir::Instruction::BinOp {{ op: ir::InfixOp::Shr, lhs: {}, rhs: {}, dst: {} }})",
                    translate_ir_arg(arg1),
                    translate_ir_arg(arg2),
                    translate_ir_register(dest))
        }
        IrPattern::And(ref dest, ref arg1, ref arg2) => {
            format!("IrLine::Instruction(&ir::Instruction::BinOp {{ op: ir::InfixOp::And, lhs: {}, rhs: {}, dst: {} }})",
                    translate_ir_arg(arg1),
                    translate_ir_arg(arg2),
                    translate_ir_register(dest))
        }
        IrPattern::Or(ref dest, ref arg1, ref arg2) => {
            format!("IrLine::Instruction(&ir::Instruction::BinOp {{ op: ir::InfixOp::Or, lhs: {}, rhs: {}, dst: {} }})",
                    translate_ir_arg(arg1),
                    translate_ir_arg(arg2),
                    translate_ir_register(dest))
        }
        IrPattern::Xor(ref dest, ref arg1, ref arg2) => {
            format!("IrLine::Instruction(&ir::Instruction::BinOp {{ op: ir::InfixOp::Xor, lhs: {}, rhs: {}, dst: {} }})",
                    translate_ir_arg(arg1),
                    translate_ir_arg(arg2),
                    translate_ir_register(dest))
        }
        IrPattern::Neg(ref dest, ref arg) => {
            format!("IrLine::Instruction(&ir::Instruction::UnOp {{ op: ir::PrefixOp::Neg, item: {}, dst: {} }})",
                    translate_ir_arg(arg),
                    translate_ir_register(dest))
        }
        IrPattern::Not(ref dest, ref arg) => {
            format!("IrLine::Instruction(&ir::Instruction::UnOp {{ op: ir::PrefixOp::Not, item: {}, dst: {} }})",
                    translate_ir_arg(arg),
                    translate_ir_register(dest))
        }
        IrPattern::CmpLt(ref dest, ref arg1, ref arg2) => {
            format!("IrLine::Instruction(&ir::Instruction::Cmp {{ cmp: ir::CmpOp::Lt, lhs: {}, rhs: {}, dst: {} }})",
                    translate_ir_arg(arg1),
                    translate_ir_arg(arg2),
                    translate_ir_register(dest))
        }
        IrPattern::CmpLe(ref dest, ref arg1, ref arg2) => {
            format!("IrLine::Instruction(&ir::Instruction::Cmp {{ cmp: ir::CmpOp::Le, lhs: {}, rhs: {}, dst: {} }})",
                    translate_ir_arg(arg1),
                    translate_ir_arg(arg2),
                    translate_ir_register(dest))
        }
        IrPattern::CmpEq(ref dest, ref arg1, ref arg2) => {
            format!("IrLine::Instruction(&ir::Instruction::Cmp {{ cmp: ir::CmpOp::Eq, lhs: {}, rhs: {}, dst: {} }})",
                    translate_ir_arg(arg1),
                    translate_ir_arg(arg2),
                    translate_ir_register(dest))
        }
        IrPattern::CmpNe(ref dest, ref arg1, ref arg2) => {
            format!("IrLine::Instruction(&ir::Instruction::Cmp {{ cmp: ir::CmpOp::Ne, lhs: {}, rhs: {}, dst: {} }})",
                    translate_ir_arg(arg1),
                    translate_ir_arg(arg2),
                    translate_ir_register(dest))
        }
        IrPattern::CmpGe(ref dest, ref arg1, ref arg2) => {
            format!("IrLine::Instruction(&ir::Instruction::Cmp {{ cmp: ir::CmpOp::Ge, lhs: {}, rhs: {}, dst: {} }})",
                    translate_ir_arg(arg1),
                    translate_ir_arg(arg2),
                    translate_ir_register(dest))
        }
        IrPattern::CmpGt(ref dest, ref arg1, ref arg2) => {
            format!("IrLine::Instruction(&ir::Instruction::Cmp {{ cmp: ir::CmpOp::Gt, lhs: {}, rhs: {}, dst: {} }})",
                    translate_ir_arg(arg1),
                    translate_ir_arg(arg2),
                    translate_ir_register(dest))
        }
        IrPattern::Alloca(ref dest) => {
            format!(
                "IrLine::Instruction(&ir::Instruction::Alloca {{ dst: {} }})",
                translate_ir_register(dest)
            )
        }
        IrPattern::Load(ref dest, ref value) => {
            format!(
                "IrLine::Instruction(&ir::Instruction::Load {{ src: {}, dst: {} }})",
                translate_ir_arg(value),
                translate_ir_register(dest)
            )
        }
        IrPattern::Store(ref value, ref dest) => {
            format!(
                "IrLine::Instruction(&ir::Instruction::Store {{ src: {}, dst: {} }})",
                translate_ir_arg(value),
                translate_ir_arg(dest)
            )
        }
        IrPattern::Call(ref dest, ref name, ref args) => {
            format!(
                "IrLine::Instruction(&ir::Instruction::Call {{ name: {}, args: ref {}, dst: {} }})",
                name,
                args,
                translate_ir_register(dest)
            )
        }
    }
}

fn translate_ir_pattern_last(ir_pattern_last: &IrPatternLast) -> String {
    match *ir_pattern_last {
        IrPatternLast::Ret(Some(ref val)) => {
            format!(
                "IrLine::CFInstruction(&ir::ControlFlowInstruction::Return {{ value: Some({}) }})",
                translate_ir_arg(val)
            )
        }
        IrPatternLast::Ret(None) => {
            "IrLine::CFInstruction(&ir::ControlFlowInstruction::Return { value: None })".into()
        }
        IrPatternLast::Br(ref cond, ref conseq, ref altern) => {
            format!("IrLine::CFInstruction(&ir::ControlFlowInstruction::Branch {{ cond: {}, conseq: {}, altern: {} }})",
                    translate_ir_arg(cond),
                    translate_ir_label(conseq),
                    translate_ir_label(altern))
        }
        IrPatternLast::Jmp(ref dest) => {
            format!(
                "IrLine::CFInstruction(&ir::ControlFlowInstruction::Jump {{ dest: {} }})",
                translate_ir_label(dest)
            )
        }
    }
}

fn translate_ir_arg(arg: &IrArg) -> String {
    match *arg {
        IrArg::Register(IrRegister(reg, IrRegisterKind::Local)) => {
            format!("ir::Value::Register(ir::Register::Local({}))", reg)
        }
        IrArg::Register(IrRegister(reg, IrRegisterKind::Stack)) => {
            format!("ir::Value::Register(ir::Register::Stack({}))", reg)
        }
        IrArg::Literal(lit) => format!("ir::Value::Immediate(ir::Immediate({}))", lit),
        IrArg::Static(lit) => format!("ir::Value::Static({})", lit),
    }
}

fn translate_ir_register(arg: &IrRegister) -> String {
    match *arg {
        IrRegister(id, IrRegisterKind::Local) => format!("ir::Register::Local({})", id),
        IrRegister(id, IrRegisterKind::Stack) => format!("ir::Register::Stack({})", id),
    }
}

fn translate_ir_label(arg: &IrLabel) -> String {
    format!("ir::Label({})", arg.0)
}

fn translate_asm(asm: &[Node<AsmInstr>], types: &HashMap<Ident, IrArg>) -> String {
    let mut s = "            ".to_owned();

    let mut new_regs = Vec::new();
    asm.iter()
        .map(|instr| {
            new_regs.extend(instr.args.iter().filter_map(|arg| {
                if let AsmArg::NewRegister(name) = **arg {
                    Some(format!(
                        "let {} = Ident::from_str(\"{}\");\n            ",
                        name, name
                    ))
                } else {
                    None
                }
            }))
        })
        .all(|_| true);

    s.push_str(&new_regs.join(""));

    let instrs: Vec<_> = asm.iter().map(|i| translate_asm_instr(i, types)).collect();
    s.push_str(&instrs.join("\n            "));
    s.push_str("\n");

    s
}

fn translate_asm_instr(instr: &AsmInstr, types: &HashMap<Ident, IrArg>) -> String {
    // FIXME: What about labels?
    let args: Vec<_> = instr
        .args
        .iter()
        .map(|arg| translate_asm_arg(arg, types))
        .collect();
    format!(
        "code.emit_instruction(asm::Instruction::new(Ident::from_str(\"{}\"), vec![{}]));",
        instr.mnemonic,
        args.join(", ")
    )
}

fn translate_asm_arg(arg: &AsmArg, types: &HashMap<Ident, IrArg>) -> String {
    match *arg {
        AsmArg::Register(ref reg) => format!(
            "asm::Argument::Register(asm::Register::Machine(MachineRegister::{:?}))",
            reg
        ),
        AsmArg::StackSlot(ref reg) => {
            format!("asm::Argument::StackSlot(asm::Register::Virtual({}))", reg)
        }
        AsmArg::NewRegister(ref reg) => {
            format!("asm::Argument::Register(asm::Register::Virtual({}))", reg)
        }
        AsmArg::IrArg(ref arg) => match *types.get(arg).unwrap() {
            IrArg::Register(IrRegister(id, IrRegisterKind::Local)) => {
                format!("asm::Argument::Register(asm::Register::Virtual({}))", id)
            }
            IrArg::Register(IrRegister(id, IrRegisterKind::Stack)) => {
                format!("asm::Argument::StackSlot({})", id)
            }
            IrArg::Literal(..) => format!("asm::Argument::Immediate(machine::Word::from({}))", arg),
            IrArg::Static(..) => format!("asm::Argument::Address({})", arg),
        },
        AsmArg::Literal(ref lit) => format!("asm::Argument::Immediate({})", lit),
        AsmArg::Label(ref target) => format!("asm::Argument::Label({})", target),
        AsmArg::Indirect {
            ref size,
            ref base,
            ref index,
            ref disp,
        } => format!(
            "asm::Argument::Indirect {{ size: {}, base: {}, index: {}, disp: {} }}",
            translate_option(size.map(|s| format!("asm::OperandSize::{:?}", s))),
            translate_option(base.clone().map(|b| translate_asm_arg_register(&b))),
            translate_option(index.clone().map(|(i, f)| format!(
                "({}, {})",
                translate_asm_arg_register(&i),
                f
            )),),
            translate_option(*disp)
        ),
    }
}

fn translate_asm_arg_register(arg: &AsmArg) -> String {
    match *arg {
        AsmArg::Register(ref reg) => format!("asm::Register::Machine(MachineRegister::{:?})", reg),
        AsmArg::NewRegister(ref reg) => format!("asm::Register::Virtual({})", reg),
        AsmArg::IrArg(ref arg) => format!("asm::Register::Virtual({})", arg),
        _ => panic!("Expected a register, got {:?}", arg),
    }
}

fn translate_option<T: fmt::Display>(val: Option<T>) -> String {
    match val {
        Some(val) => format!("Some({})", val),
        None => "None".to_owned(),
    }
}
