//! The lexer: split the source into a stream of tokens

use std::borrow::ToOwned;
use std::str::CharIndices;
use driver::interner::Ident;
use driver::session;
use driver::codemap::{BytePos, Loc};
use front::ast::Spanned;
use back::instsel::rulecomp::tokens::{Token, lookup_keyword};


pub struct Lexer<'a> {
    source: &'a str,

    iter: CharIndices<'a>,
    pos: usize,
    curr: Option<char>,

    lineno: usize,
}

impl<'a> Lexer<'a> {
    // --- Lexer: The public API ------------------------------------------------

    /// Create a new lexer from a given string and file name
    pub fn new(source: &'a str, _: &'a str) -> Lexer<'a> {
        let mut iter = source.char_indices();
        let (pos, curr) = iter.next()
            .map(|(p, c)| (p, Some(c)))  // Make `curr` an Option
            .unwrap_or_else(|| (0, None));  // Set `curr` to None

        Lexer {
            source: source,

            pos: pos,
            curr: curr,

            iter: iter,

            lineno: 1,
        }
    }

    /// Get the next token
    pub fn next_token(&mut self) -> Spanned<Token> {
        while !self.is_eof() {
            // Read the next token as long as the lexer requests us to do so
            if let Some(token) = self.read_token() {
                return token;
            }
        }

        Spanned::new(Token::EOF, self.pos as u32, self.pos as u32)
    }

    // --- Lexer: Helpers -------------------------------------------------------

    /// Report a fatal error back to the user
    fn fatal(&self, msg: String) -> ! {
        fatal_at!(msg; self.get_source());
        session().abort()
    }

    /// Are we done yet?
    fn is_eof(&self) -> bool {
        self.curr.is_none()
    }

    /// Get the current source position we're at
    pub fn get_source(&self) -> Loc {
        session().codemap.resolve(BytePos(self.pos as u32))
    }

    // --- Lexer: Character processing ------------------------------------------

    /// Move along to the next character
    fn bump(&mut self) {
        if let Some((pos, curr)) = self.iter.next() {
            self.curr = Some(curr);
            self.pos = pos;
        } else {
            self.curr = None;
            self.pos = self.source.len();
        }

        debug!("Moved on to {:?}", self.curr)
    }

    /// An escaped representation of the current character
    fn curr_escaped(&self) -> String {
        match self.curr {
            Some(c) => c.escape_default().collect(),
            None    => "EOF".to_owned()
        }
    }

    /// Collect & consume all consecutive characters into a string as long as a condition is true
    fn collect<F>(&mut self, cond: F) -> &'a str
            where F: Fn(&char) -> bool
    {
        let start = self.pos;

        debug!("start colleting (start = {}, char = {:?})", start, self.curr);

        while let Some(c) = self.curr {
            if cond(&c) {
                self.bump();
            } else {
                break;
            }
        }

        debug!("colleting finished (pos = {})", self.pos);

        &self.source[start..self.pos]
    }

    /// Consume all consecutive characters matching a condition
    fn eat_all<F>(&mut self, cond: F)
            where F: Fn(&char) -> bool {
        while let Some(c) = self.curr {
            if cond(&c) { self.bump(); }
            else { break; }
        }
    }

    // --- Lexer: Tokenizers ----------------------------------------------------

    /// Skip over a comment string
    fn skip_comment(&mut self) {
        self.eat_all(|c| *c != '\n');
    }

    pub fn tokenize_snippet(&mut self) -> Ident {
        debug!("tokenizing a snippet");

        let start = self.pos;
        self.collect(|c| *c != '}');
        self.bump();

        let rust_code = &self.source[start + 1..self.pos - 1];

        return Ident::new(rust_code);
    }

    /// Tokenize an identifier
    fn tokenize_ident(&mut self) -> Token {
        debug!("tokenizing an ident");

        let ident = self.collect(|c| {
            c.is_alphabetic() || c.is_numeric() || *c == '_'
        });

        // Check whether it's a keyword or an identifier
        if let Some(kw) = lookup_keyword(&ident) {
            Token::Keyword(kw)
        } else {
            Token::Ident(Ident::new(ident))
        }
    }

    /// Tokenize a literal
    fn tokenize_literal(&mut self) -> Token {
        debug!("tokenizing a literal");

        let literal = self.collect(|c| {
            c.is_alphabetic() || c.is_numeric() || *c == '_'
        });

        Token::Literal(Ident::new(literal))
    }

    /// Read the next token and return it
    ///
    /// If `None` is returned, the current token is to be ignored and the
    /// lexer requests the reader to read the next token instead.
    ///
    /// Precondition: self.curr is not None
    fn read_token(&mut self) -> Option<Spanned<Token>> {
        macro_rules! emit(
            ($_self:ident, $( next: $ch:expr => $tok:expr ),* ; default: $default:expr) => (
                {
                    $_self.bump();
                    match $_self.curr {
                        $( Some($ch) => { $_self.bump(); $tok } , )*
                        _ => $default
                    }
                }
            );

            ($_self:ident, $token:expr) => (
                {
                    $_self.bump();
                    $token
                }
            );
        );

        debug!("tokenizing with current character = `{}` at {}",
                 self.curr_escaped(), self.pos);

        let c = self.curr.unwrap();
        let lo = self.pos;

        let token: Token = match c {
            '!' => emit!(self, Token::Bang),

            '$' => emit!(self, Token::Dollar),

            '%' => emit!(self, Token::Percent),

            '0' => emit!(self, Token::Zero),

            '=' => emit!(self, next: '>' => Token::FatArrow;
                               default: Token::Equal),

            '+' => emit!(self, Token::Plus),

            '-' => emit!(self, next: '>' => Token::Arrow;
                               default: Token::Minus),

            '*' => emit!(self, Token::Asterisk),

            '.' => emit!(self, next: '.' => Token::DoubleDot;
                               default: Token::Dot),

            '(' => emit!(self, Token::LParen),

            ')' => emit!(self, Token::RParen),

            '[' => emit!(self, Token::LBracket),

            ']' => emit!(self, Token::RBracket),

            '{' => emit!(self, Token::LBrace),

            '}' => emit!(self, Token::RBrace),

            ',' => emit!(self, Token::Comma),

            ';' => emit!(self, Token::Semicolon),

            '/' => emit!(self, next: '/' => { self.skip_comment(); return None };
                               default: self.fatal(format!("unexpected character: `{}`", c))),

            c if c.is_alphabetic() => self.tokenize_ident(),

            c if c.is_numeric() => self.tokenize_literal(),

            c if c.is_whitespace() => {
                // Skip whitespaces of any type
                if c == '\n' {
                    self.lineno += 1;
                    //let offset = if self.nextch() == Some('\r') { 2 } else { 1 };
                    session().codemap.new_line(BytePos(self.pos as u32))
                }

                self.bump();
                return None;
            },
            c => self.fatal(format!("unexpected character: `{}`", c))
        };

        debug!("emitted token: `{:?}`", token);

        Some(Spanned::new(token, lo as u32, self.pos as u32))
    }
}