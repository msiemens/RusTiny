use std::borrow::ToOwned;
use ast::{BinOp, UnOp};
use driver;
use front::tokens::{Token, lookup_keyword};
use util::fatal;


pub struct Lexer<'a> {
    source: &'a str,
    file: &'a str,
    len: usize,

    pos: usize,
    curr: Option<char>,

    lineno: usize
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str, file: &'a str) -> Lexer<'a> {
        Lexer {
            source: source,
            file: file,
            len: source.len(),

            pos: 0,
            curr: Some(source.char_at(0)),

            lineno: 1
        }
    }

    // --- Lexer: Helpers -------------------------------------------------------

    fn fatal(&self, msg: String) -> ! {
        fatal(msg, self.get_source())
    }


    fn is_eof(&self) -> bool {
        self.curr.is_none()
    }

    pub fn get_source(&self) -> usize {
        self.lineno
    }

    // --- Lexer: Character processing ------------------------------------------

    fn bump(&mut self) {
        self.curr = self.nextch();
        self.pos += 1;

        debug!("Moved on to {:?}", self.curr)
    }

    fn nextch(&self) -> Option<char> {
        let mut new_pos = self.pos + 1;

        // When encountering multi-byte UTF-8, we may stop in the middle
        // of it. Fast forward till we see the next actual char or EOF

        while !self.source.is_char_boundary(new_pos)
                && self.pos < self.len {
            new_pos += 1;
        }

        if new_pos < self.len {
            Some(self.source.char_at(new_pos))
        } else {
            None
        }
    }

    fn curr_repr(&self) -> String {
        match self.curr {
            Some(c) => c.escape_default().collect(),
            None    => "EOF".to_owned()
        }
    }

    fn expect(&mut self, expect: char) {
        if self.curr != Some(expect) {
            // Build error message
            let expect_str = match expect {
                '\'' => String::from_str("quote"),
                c    => format!("'{}'", c)
            };
            let found_str = match self.curr {
                Some(_) => format!("'{}'", self.curr_repr()),
                None    => String::from_str("EOF")
            };

            self.fatal(format!("Expected `{}`, found `{}`",
                               expect_str, found_str))
        }

        self.bump();
    }

    fn collect<F>(&mut self, cond: F) -> &'a str
            where F: Fn(&char) -> bool {
        let start = self.pos;

        debug!("start colleting");

        while let Some(c) = self.curr {
            if cond(&c) {
                self.bump();
            } else {
                debug!("colleting finished");
                break;
            }
        }

        let end = self.pos;

        &self.source[start..end]
    }

    fn eat_all<F>(&mut self, cond: F)
            where F: Fn(&char) -> bool {
        while let Some(c) = self.curr {
            if cond(&c) { self.bump(); }
            else { break; }
        }
    }

    // --- Lexer: Tokenizers ----------------------------------------------------

    fn tokenize_ident(&mut self) -> Token {
        debug!("Tokenizing an ident");

        let ident = self.collect(|c| {
            c.is_alphabetic() || c.is_numeric() || *c == '_'
        });

        if let Some(kw) = lookup_keyword(&ident) {
            Token::Keyword(kw)
        } else {
            Token::Ident(driver::get_interner().intern(ident))
        }
    }

    fn tokenize_digit(&mut self) -> Token {
        debug!("Tokenizing a digit");

        let integer_str = self.collect(|c| c.is_numeric());
        let integer     = match integer_str.parse() {
            Ok(i) => i,
            Err(_) => self.fatal(format!("invalid integer: {}", integer_str))
        };

        Token::Int(integer)
    }

    fn tokenize_char(&mut self) -> Token {
        debug!("Tokenizing a char");

        self.bump();  // '\'' matched, move on

        let c = self.curr.unwrap_or_else(|| {
            self.fatal(format!("expected a char, found EOF"));
        });
        let tok = if c == '\\' {
            // Escaped char, let's take a look on one more char
            self.bump();
            match self.curr {
                Some('n')  => Token::Char('\n'),
                Some('\'') => Token::Char('\''),
                Some(c) => self.fatal(format!("unsupported or invalid escape sequence: \\{}", c)),
                None => self.fatal(format!("expected escaped char, found EOF"))
            }
        } else {
            Token::Char(c)
        };
        self.bump();

        // Match closing quote
        self.expect('\'');

        tok
    }

    /// Read the next token and return it
    ///
    /// If `None` is returned, the current token is to be ignored and the
    /// lexer requests the reader to read the next token instead.
    fn read_token(&mut self) -> Option<Token> {
        let c = match self.curr {
            Some(c) => c,
            None    => return Some(Token::EOF)
        };

        let token = match c {
            // TODO: Comments & macro
            // Binops
            '+' => {
                self.bump();
                Token::BinOp(BinOp::Add)
            },
            '-' => {
                self.bump();
                if self.curr == Some('>') {
                    self.bump();
                    Token::RArrow
                } else {
                    Token::BinOp(BinOp::Sub)
                }
            },
            '*' => {
                self.bump();
                if self.curr == Some('*') {
                    self.bump();
                    Token::BinOp(BinOp::Pow)
                } else {
                    Token::BinOp(BinOp::Mul)
                }
            },
            '/' => {
                self.bump();
                if self.curr == Some('/') {
                    self.eat_all(|c| *c != '\n');
                    return None;
                } else {
                    Token::BinOp(BinOp::Div)
                }
            },
            '%' => {
                self.bump();
                Token::BinOp(BinOp::Mod)
            },
            '&' => {
                self.bump();
                if self.curr == Some('&') {
                    self.bump();
                    Token::BinOp(BinOp::And)
                } else {
                    Token::BinOp(BinOp::BitAnd)
                }
            },
            '|' => {
                self.bump();
                if self.curr == Some('|') {
                    self.bump();
                    Token::BinOp(BinOp::Or)
                } else {
                    Token::BinOp(BinOp::BitOr)
                }
            },
            '^' => {
                self.bump();
                Token::BinOp(BinOp::BitXor)
            },
            '<' => {
                self.bump();
                match self.curr {
                    Some('<') => { self.bump(); Token::BinOp(BinOp::Shl) },
                    Some('=') => { self.bump(); Token::BinOp(BinOp::Le) },
                    _ => Token::BinOp(BinOp::Lt)
                }
            },
            '>' => {
                self.bump();
                match self.curr {
                    Some('>') => { self.bump(); Token::BinOp(BinOp::Shr) },
                    Some('=') => { self.bump(); Token::BinOp(BinOp::Ge) },
                    _ => Token::BinOp(BinOp::Gt)
                }
            },
            '=' => {
                self.bump();
                if self.curr == Some('=') {
                    self.bump();
                    Token::BinOp(BinOp::EqEq)
                } else {
                    Token::Eq
                }
            },
            '!' => {
                self.bump();
                if self.curr == Some('=') {
                    self.bump();
                    Token::BinOp(BinOp::Ne)
                } else {
                    Token::UnOp(UnOp::Not)
                }
            },
            '(' => { self.bump(); Token::LParen },
            ')' => { self.bump(); Token::RParen },
            '{' => { self.bump(); Token::LBrace },
            '}' => { self.bump(); Token::RBrace },
            ',' => { self.bump(); Token::Comma },
            ':' => { self.bump(); Token::Colon },
            ';' => { self.bump(); Token::Semicolon },
            c if c.is_alphabetic()  => self.tokenize_ident(),
            c if c.is_numeric()     => self.tokenize_digit(),
            '\''                    => self.tokenize_char(),
            c if c.is_whitespace() => {
                if c == '\n' { self.lineno += 1; }

                self.bump();
                return None;
            },
            c => {
                self.fatal(format!("unknown token: {}", c))
                // UNKNOWN(format!("{}", c).into_string())
            }
        };

        Some(token)
    }

    pub fn next_token(&mut self) -> Token {
        if self.is_eof() {
            Token::EOF
        } else {
            // Read the next token until it's not none
            loop {
                if let Some(token) = self.read_token() {
                    return token;
                }
            }
        }
    }

    #[allow(dead_code)]  // Used for tests
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = vec![];

        while !self.is_eof() {
            debug!("Processing {:?}", self.curr);

            if let Some(t) = self.read_token() {
                tokens.push(t);
            }

            debug!("So far: {:?}", tokens)
        }

        tokens
    }
}