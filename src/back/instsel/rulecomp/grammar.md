# AST

```ignore
rules:      kw_rules BANG LBRACE rule (, rule)+ RBRACKET
rule:       pattern RFATARROW impl
pattern:    LBRACKET ir (SEMICOLON ir)* (SEMICOLON DOUBLEDOT | SEMICOLON ir_last) RBRACKET
impl:       LBRACE (asm SEMICOLON)+ RBRACE

ir:         ir_dst kw_add ir_arg COMMA ir_arg
          | ir_dst kw_sub ir_arg COMMA ir_arg
          | ir_dst kw_mul ir_arg COMMA ir_arg
          | ir_dst kw_div ir_arg COMMA ir_arg
          | ir_dst kw_pow ir_arg COMMA ir_arg
          | ir_dst kw_mod ir_arg COMMA ir_arg
          | ir_dst kw_shl ir_arg COMMA ir_arg
          | ir_dst kw_shr ir_arg COMMA ir_arg
          | ir_dst kw_and ir_arg COMMA ir_arg
          | ir_dst kw_or  ir_arg COMMA ir_arg
          | ir_dst kw_xor ir_arg COMMA ir_arg
          | ir_dst kw_neg ir_arg
          | ir_dst kw_cmp kw_lt ir_arg COMMA ir_arg
          | ir_dst kw_cmp kw_le ir_arg COMMA ir_arg
          | ir_dst kw_cmp kw_eq ir_arg COMMA ir_arg
          | ir_dst kw_cmp kw_ne ir_arg COMMA ir_arg
          | ir_dst kw_cmp kw_ge ir_arg COMMA ir_arg
          | ir_dst kw_cmp kw_gt ir_arg COMMA ir_arg
          | ir_dst kw_alloca
          | ir_dst kw_load ir_arg_address
          | kw_store ir_arg COMMA ir_arg_address
          | kw_call IDENT LBRACKET ir_arg_address (COMMA ir_arg_address)* RBRACKET

ir_last:    kw_ret (ir_arg)?
          | kw_br ir_arg IDENT IDENT
          | kw_jmp IDENT
ir_dst:             ir_arg_address EQ      # %(...) =
ir_arg:             ir_arg_address | ir_arg_literal
ir_arg_address:     PERCENT LPAREN ident RPAREN | LBRACE ident RBRACE    # %(...) | {...}
ir_arg_literal:     ZERO LPAREN ident RPAREN            # 0(...)

asm:        mnemonic asm_arg (, asm_arg)* SEMICOLON
asm_arg:    asm_register
          | asm_ir_arg
          | asm_stack_slot
          | asm_new_register
          | asm_literal
asm_register:     asm_ref_size "ptr" asm_ref | asm_register_name
asm_register_name: kw_rax | ... | kw_rbp
asm_ref:          LBRACKET asm_register_name ( ('+' | '-') asm_register_name '*' asm_literal )? ( ('+' | '-') asm_literal )? RBRACKET
asm_ref_size:     word | dword | qword
asm_ir_arg:       DOLLAR IDENT
asm_stack_slot:  LBRACKET LBRACKET IDENT RBRACKET RBRACKET
asm_new_register: PERCENT LBRACE IDENT RBRACE
asm_literal:      0-9 (0-9|a-Z)*
```

// Intel syntax:
// https://github.com/llvm-mirror/llvm/blob/master/lib/Target/X86/AsmParser/X86AsmParser.cpp#L1501-L1559
// https://sourceware.org/git/gitweb.cgi?p=binutils-gdb.git;a=blob;f=gas/config/tc-i386-intel.c;h=cff0ae7878cc55f1d05d67a44769d75fc729c7c7;hb=HEAD#l823
// https://github.com/sporst/Reverse-Engineering-Scripts/blob/master/antlr_x86/x86.g#L237-L256
// http://www.c-jump.com/CIS77/ASM/Addressing/lecture.html
// Intel Architecture Manual Vol 1, 3.7 Operand Addressing