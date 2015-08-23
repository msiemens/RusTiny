mod ast;
mod tokens;
mod lexer;
mod parser;


pub fn compile_rules(input: &str, filename: &str) -> String {
    let mut parser = parser::Parser::new(lexer::Lexer::new(input, filename));
    let rules = parser.parse();

    println!("Rules: {:#?}", rules);

    format!("# TODO: Description
# Use declarations
pub fn trans_instr(instr: &mut [ir::Instruction],
                   last: &ir::ControlFlowInstruction,
                   code: &mut machine::MachineCode)
                   -> usize
{{
    match instr {{
{}
    }}
}}
", rules.iter().map(|r| translate_rule(r)).collect::<Vec<_>>().join(",\n"))
}

fn translate_rule(rule: &ast::Rule) -> String {
    "        ".to_owned()
}