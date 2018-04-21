use driver::interner::Ident;
use driver::session;
use front::ast::{Node, Span};
use back::machine::MachineRegister;
use back::machine::asm::OperandSize;
use back::instsel::rulecomp::ast::*;
use back::instsel::rulecomp::lexer::Lexer;
use back::instsel::rulecomp::tokens::{Token, Keyword};


pub struct Parser<'a> {
    token: Token,
    span: Span,
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    // --- The public API -------------------------------------------------------

    /// Create a new parser instance
    pub fn new(mut lx: Lexer<'a>) -> Parser<'a> {
        // Initialize with first token
        let first_token = lx.next_token();

        Parser {
            token: first_token.value,
            span: first_token.span,
            lexer: lx
        }
    }

    /// Process all tokens and create an AST
    pub fn parse(&mut self) -> Vec<Node<Rule>> {
        // Grammar: kw_rules BANG LBRACE rule+ RBRACKET
        debug!("Starting parsing");
        //let lo = self.span;

        let mut rules = Vec::new();

        self.expect(Token::Keyword(Keyword::Rules));
        self.expect(Token::Bang);
        self.expect(Token::LBrace);

        while self.token != Token::RBrace {
            rules.push(self.parse_rule());

            if !self.eat(Token::Comma) {
                break
            }
        }

        self.expect(Token::RBrace);

        debug!("Parsing finished");

        rules
    }

    // --- Error handling -------------------------------------------------------

    /// Stop compiling because of a fatal error
    fn fatal<S: AsRef<str>>(&self, msg: S) -> ! {
        fatal_at!(msg; self.span);
        session().abort()
    }

    /// Stop compiling because of an unexpected token
    fn unexpected_token(&self, expected: Option<&'static str>) -> ! {
        match expected {
            Some(ex) => self.fatal(format!("unexpected token: `{}`, expected {}",
                                   &self.token, ex)),
            None => self.fatal(format!("unexpected token: `{}`", &self.token))
        }
    }

    // --- Token processing -----------------------------------------------------

    /// Move along to the next token
    fn bump(&mut self) {
        let next_token = self.lexer.next_token();
        self.token = next_token.value;
        self.span = next_token.span;

        debug!("next token: `{:?}`, span: {:?}`", self.token, self.span);
    }

    /// Try consuming a token, return `true` on succes
    fn eat(&mut self, tok: Token) -> bool {
        if self.token == tok {
            self.bump();
            true
        } else {
            false
        }
    }

    /// Try consuming a token, quit with a fatal error otherwise
    fn expect(&mut self, tok: Token) {
        if !self.eat(tok) {
            self.fatal(format!("expected `{}`, found `{}`", tok, self.token))
        }
    }

    // --- Parse tokens ---------------------------------------------------------

    /// Parse an identifier
    fn parse_ident(&mut self) -> Node<Ident> {
        debug!("parsing an ident");

        let span = self.span;
        let ident = match self.token {
            Token::Ident(id) => id,
            _ => self.unexpected_token(Some("an identifier"))
        };
        self.bump();

        Node::new(ident, span)
    }

    // --- Parse ir patterns ----------------------------------------------------

    fn on_last_pattern(&self) -> bool {
        match self.token {
            Token::Keyword(Keyword::Ret)
            | Token::Keyword(Keyword::Br)
            | Token::Keyword(Keyword::Jmp) => true,
            _ => false
        }
    }

    fn parse_ir_pattern(&mut self) -> Node<IrPattern> {
        // Grammar: not funny
        macro_rules! binop {
            ($dst:ident, $pat:path) => {
                {
                    self.bump();
                    let arg1 = self.parse_ir_arg();
                    self.expect(Token::Comma);
                    let arg2 = self.parse_ir_arg();

                    $pat($dst, arg1, arg2)
                }
            }
        }

        debug!("parsing an ir pattern");
        let lo = self.span;

        let pattern = if self.eat(Token::Keyword(Keyword::Store)) {
            // store ..., %(...)
            let src = self.parse_ir_arg();
            self.expect(Token::Comma);
            let dst = self.parse_ir_arg();
            IrPattern::Store(src, dst)
        } else {
            // %(`dest`) = ...
            let dst = self.parse_ir_register();
            self.expect(Token::Equal);

            match self.token {
                Token::Keyword(Keyword::Add) => binop!(dst, IrPattern::Add),
                Token::Keyword(Keyword::Sub) => binop!(dst, IrPattern::Sub),
                Token::Keyword(Keyword::Mul) => binop!(dst, IrPattern::Mul),
                Token::Keyword(Keyword::Div) => binop!(dst, IrPattern::Div),
                Token::Keyword(Keyword::Pow) => binop!(dst, IrPattern::Pow),
                Token::Keyword(Keyword::Mod) => binop!(dst, IrPattern::Mod),
                Token::Keyword(Keyword::Shl) => binop!(dst, IrPattern::Shl),
                Token::Keyword(Keyword::Shr) => binop!(dst, IrPattern::Shr),
                Token::Keyword(Keyword::And) => binop!(dst, IrPattern::And),
                Token::Keyword(Keyword::Or)  => binop!(dst, IrPattern::Or),
                Token::Keyword(Keyword::Xor) => binop!(dst, IrPattern::Xor),
                Token::Keyword(Keyword::Neg) => {
                    self.bump();
                    IrPattern::Neg(dst, self.parse_ir_arg())
                },
                Token::Keyword(Keyword::Not) => {
                    self.bump();
                    IrPattern::Not(dst, self.parse_ir_arg())
                },
                Token::Keyword(Keyword::Cmp) => {
                    self.bump();

                    match self.token {
                        Token::Keyword(Keyword::Lt) => binop!(dst, IrPattern::CmpLt),
                        Token::Keyword(Keyword::Le) => binop!(dst, IrPattern::CmpLe),
                        Token::Keyword(Keyword::Eq) => binop!(dst, IrPattern::CmpEq),
                        Token::Keyword(Keyword::Ne) => binop!(dst, IrPattern::CmpNe),
                        Token::Keyword(Keyword::Ge) => binop!(dst, IrPattern::CmpGe),
                        Token::Keyword(Keyword::Gt) => binop!(dst, IrPattern::CmpGt),
                        _ => self.fatal(format!("Invalid comparison: {}", self.token))
                    }
                },
                Token::Keyword(Keyword::Alloca) => {
                    self.bump();
                    IrPattern::Alloca(dst)
                },
                Token::Keyword(Keyword::Load) => {
                    self.bump();
                    let val = self.parse_ir_arg();
                    IrPattern::Load(dst, val)
                },
                Token::Keyword(Keyword::Call) => {
                    self.bump();

                    let func = self.parse_ident();

                    self.expect(Token::LBracket);

                    let args = self.parse_ident();

                    self.expect(Token::DoubleDot);
                    self.expect(Token::RBracket);

                    IrPattern::Call(dst, func, args)
                },
                _ => self.unexpected_token(Some("an ir keyword"))
            }

        };

        Node::new(pattern, lo + self.span)
    }

    fn parse_ir_pattern_last(&mut self) -> Node<IrPatternLast> {
        debug!("parsing a last ir pattern");
        let lo = self.span;

        let last = match self.token {
            Token::Keyword(Keyword::Ret) => {
                self.bump();

                let val = if self.eat(Token::Keyword(Keyword::Void)) {
                    None
                } else {
                    Some(self.parse_ir_arg())
                };

                IrPatternLast::Ret(val)
            },
            Token::Keyword(Keyword::Br) => {
                self.bump();
                let cond = self.parse_ir_arg();
                self.expect(Token::Comma);
                let conseq = self.parse_ir_label();
                self.expect(Token::Comma);
                let altern = self.parse_ir_label();

                IrPatternLast::Br(cond, conseq, altern)
            },
            Token::Keyword(Keyword::Jmp) => {
                self.bump();
                IrPatternLast::Jmp(self.parse_ir_label())
            },
            _ => self.unexpected_token(None)
        };

        Node::new(last, lo + self.span)
    }

    fn parse_ir_register(&mut self) -> Node<IrRegister> {
        debug!("parsing an ir register");
        let lo = self.span;

        let ident;
        let kind;

        match self.token {
            Token::Percent => {
                kind = IrRegisterKind::Local;

                self.bump();
                self.expect(Token::LParen);
                ident = self.parse_ident();
                self.expect(Token::RParen);
            },

            Token::LBrace => {
                kind = IrRegisterKind::Stack;

                self.bump();
                ident = self.parse_ident();
                self.expect(Token::RBrace);
            },

            tok => panic!("Unexpected token: {}, expected PERCENT or LBrace", tok)
        }

        Node::new(IrRegister(*ident, kind), lo + self.span)
    }

    fn parse_ir_label(&mut self) -> Node<IrLabel> {
        debug!("parsing an ir label arg");
        let lo = self.span;

        let ident = self.parse_ident();

        Node::new(IrLabel(*ident), lo + self.span)
    }

    fn parse_ir_arg(&mut self) -> Node<IrArg> {
        debug!("parsing an ir argument");

        let lo = self.span;

        let arg = match self.token {
            Token::Percent | Token::LBrace => IrArg::Register(*self.parse_ir_register()),
            Token::Zero => {
                self.bump();
                self.expect(Token::LParen);
                let ident = self.parse_ident();
                self.expect(Token::RParen);

                IrArg::Literal(*ident)
            },
            Token::At => {
                self.bump();
                self.expect(Token::LParen);
                let ident = self.parse_ident();
                self.expect(Token::RParen);

                IrArg::Static(*ident)
            },
            _ => self.unexpected_token(Some("one of '%' | '0'"))
        };

        Node::new(arg, lo + self.span)
    }

    // --- Parse asm instructions -----------------------------------------------

    fn parse_asm_instruction(&mut self) -> Node<AsmInstr> {
        // Grammar: mnemonic asm_arg (, asm_arg)* SEMICOLON
        debug!("parsing an asm instruction");
        let lo = self.span;

        let mut args = Vec::new();

        let mnemonic = match self.token {
            Token::Ident(..) => self.parse_ident(),
            Token::Keyword(kw) => {
                // Some mnemonics have the same name as some IR keywords,
                // thus we need to extract them
                self.bump();
                Node::new(Ident::from_str(&format!("{}", kw)), lo + self.span)
            },
            _ => self.unexpected_token(Some("a mnemonic"))
        };

        while self.token != Token::Semicolon {
            args.push(self.parse_asm_arg());

            if !self.eat(Token::Comma) {
                break
            }
        }

        Node::new(AsmInstr { mnemonic, args }, lo + self.span)
    }

    fn parse_asm_arg(&mut self) -> Node<AsmArg> {
        // Grammar: mnemonic asm_arg (, asm_arg)* SEMICOLON
        debug!("parsing an asm argument");
        let lo = self.span;

        let arg = match self.token {
            Token::Dollar => {
                self.bump();
                AsmArg::IrArg(self.parse_ident())
            },
            Token::LBrace => {
                self.bump();
                let ident = self.parse_ident();
                self.expect(Token::RBrace);

                AsmArg::StackSlot(ident)
            },
            Token::Percent => {
                self.bump();
                self.expect(Token::LParen);
                let ident = self.parse_ident();
                self.expect(Token::RParen);

                AsmArg::NewRegister(ident)
            },
            Token::Dot => {
                self.bump();

                AsmArg::Label(self.parse_ident())
            },
            Token::Literal(literal) => {
                self.bump();

                AsmArg::Literal(Node::new(literal, lo + self.span))
            },
            Token::LBracket => {
                self.parse_asm_memory_operand(None)
            }
            Token::Keyword(Keyword::Byte) => {
                self.bump();
                self.expect(Token::Keyword(Keyword::Ptr));
                self.parse_asm_memory_operand(Some(OperandSize::Byte))
            },
            Token::Keyword(Keyword::Word) => {
                self.bump();
                self.expect(Token::Keyword(Keyword::Ptr));
                self.parse_asm_memory_operand(Some(OperandSize::Word))
            },
            Token::Keyword(Keyword::DWord) => {
                self.bump();
                self.expect(Token::Keyword(Keyword::Ptr));
                self.parse_asm_memory_operand(Some(OperandSize::DWord))
            },
            Token::Keyword(Keyword::QWord) => {
                self.bump();
                self.expect(Token::Keyword(Keyword::Ptr));
                self.parse_asm_memory_operand(Some(OperandSize::QWord))
            },
            _ => AsmArg::Register(self.parse_asm_register())
        };

        Node::new(arg, lo + self.span)
    }

    fn parse_asm_register(&mut self) -> MachineRegister {
        let ident = self.parse_ident();
        match &*ident.to_lowercase() {
            "rax" => MachineRegister::RAX,
            "rbx" => MachineRegister::RBX,
            "rcx" => MachineRegister::RCX,
            "rdx" => MachineRegister::RDX,
            "rsi" => MachineRegister::RSI,
            "rdi" => MachineRegister::RDI,
            "r8"  => MachineRegister::R8,
            "r9"  => MachineRegister::R9,
            "r10" => MachineRegister::R10,
            "r11" => MachineRegister::R11,
            "r12" => MachineRegister::R12,
            "r13" => MachineRegister::R13,
            "r14" => MachineRegister::R14,
            "r15" => MachineRegister::R15,
            "rsp" => MachineRegister::RSP,
            "rbp" => MachineRegister::RBP,
            "cl"  => MachineRegister::CL,
            _ => self.fatal(format!("Invalid register: {}", ident))
        }
    }

    fn parse_asm_memory_operand(&mut self, size: Option<OperandSize>) -> AsmArg {
        self.expect(Token::LBracket);

        let mut base = None;
        let mut index = None;
        let mut disp = None;

        // Parse base or index
        match self.token {
            Token::Dollar | Token::Percent | Token::Ident(..) => {
                let reg = self.parse_asm_arg().unwrap();
                if let Token::Asterisk = self.token {
                    self.expect(Token::Asterisk);
                    let scale = if let Token::Literal(lit) = self.token {
                        lit.parse::<u32>().unwrap()
                    } else {
                        self.unexpected_token(Some("a numeric literal"));
                    };
                    self.bump();

                    index = Some((Box::new(reg), scale));
                } else {
                    base = Some(Box::new(reg));
                }
            },
            _ => {
                self.unexpected_token(Some("a register"));
            }
        }

        // If the previous part was the base, now try to parse the index
        if self.eat(Token::Plus) && index.is_none() {
            match self.token {
                Token::Ident(..) | Token::Dollar | Token::Percent => {
                    let reg = self.parse_asm_arg().unwrap();
                    self.expect(Token::Asterisk);
                    let scale = if let Token::Literal(lit) = self.token {
                        lit.parse::<u32>().unwrap()
                    } else {
                        self.unexpected_token(Some("a numeric literal"));
                    };
                    self.bump();

                    index = Some((Box::new(reg), scale));
                },
                _ => {}  // Propably the displacement
            }
        }

        // Parse displacement
        match self.token {
            Token::Literal(lit) => {
                disp = Some(lit.parse().unwrap())
            },
            Token::Minus => {
                self.bump();

                disp = if let Token::Literal(lit) = self.token {
                    Some(-lit.parse::<i32>().unwrap())
                } else {
                    self.unexpected_token(Some("a numeric literal"));
                };

                self.bump();
            },
            _ => {}
        }

        self.expect(Token::RBracket);

        AsmArg::Indirect { size, base, index, disp }
    }

    // --- Parse patterns & impls -----------------------------------------------

    fn parse_pattern(&mut self) -> Node<Pattern> {
        // Grammar: LBRACKET ir (SEMICOLON ir)* (SEMICOLON DOUBLEDOT | SEMICOLON ir_last) RBRACKET
        debug!("parsing a pattern");
        let lo = self.span;

        let mut patterns = Vec::new();
        let mut last = None;

        self.expect(Token::LBracket);

        loop {
            if self.eat(Token::DoubleDot) {
                break
            } else if self.on_last_pattern() {
                last = Some(self.parse_ir_pattern_last());
                break
            } else {
                patterns.push(self.parse_ir_pattern());
                self.expect(Token::Semicolon);
            }
        }

        self.expect(Token::RBracket);

        let cond = if self.eat(Token::Keyword(Keyword::If)) {
            let snippet = self.lexer.tokenize_snippet();
            self.bump();

            Some(snippet)
        } else {
            None
        };

        Node::new(Pattern { ir_patterns: patterns, last,  cond }, lo + self.span)
    }

    fn parse_rust_impl(&mut self) -> Node<Ident> {
        // Grammar: LBRACE <Rust code> RBRACE
        debug!("parsing a rust impl");
        let lo = self.span;

        let snippet = self.lexer.tokenize_snippet();
        self.bump();

        Node::new(snippet, lo + self.span)
    }

    fn parse_asm_impl(&mut self) -> Node<Vec<Node<AsmInstr>>> {
        // Grammar: LBRACE (asm SEMICOLON)+ RBRACE
        debug!("parsing an asm impl");
        let lo = self.span;

        let mut instructions = Vec::new();

        self.expect(Token::LBrace);

        while self.token != Token::RBrace {
            instructions.push(self.parse_asm_instruction());
            self.expect(Token::Semicolon);
        }

        self.expect(Token::RBrace);

        Node::new(instructions, lo + self.span)
    }

    // --- Parse rules ----------------------------------------------------------

    fn parse_rule(&mut self) -> Node<Rule> {
        // Grammar: pattern RFATARROW impl
        debug!("parsing a single rule");
        let lo = self.span;

        let pattern = self.parse_pattern();

        let implementation = if self.eat(Token::Arrow) {
            Impl::Rust(self.parse_rust_impl())
        } else if self.eat(Token::FatArrow) {
            Impl::Asm(self.parse_asm_impl())
        } else {
            self.unexpected_token(None)
        };

        Node::new(Rule { pattern, implementation }, lo + self.span)
    }
}