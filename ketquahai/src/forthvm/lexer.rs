// ============================================================
// lexer.rs -- Bộ phân tích từ tố (Stage 2: Full Julia syntax)
// ~35 token types
// ============================================================

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    Eof, Num, HexNum, Ident, StringLit, Newline,
    // Phép gán
    Assign, PlusEq, MinusEq, StarEq, SlashEq, PercentEq,
    // Toán tử số học
    Plus, Minus, Star, Slash, Percent, Caret, // ^ = power
    // Toán tử so sánh
    Eq, Neq, Gt, Lt, Gte, Lte,
    // Toán tử logic
    And, Or, Not,
    // Bitwise
    Ampersand, Pipe, Tilde, Shl, Shr,
    // Dấu ngoặc
    LParen, RParen, LBracket, RBracket,
    // Dấu câu
    Comma, Semicolon, Colon, Arrow, Question, Dollar,
    // Từ khoá
    If, Else, ElseIf, End, While, For, Function, Return,
    Println, Print, Break, Continue, True, False,
    Local, Include,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub num_val: i32,
    pub str_val: String,
    pub col: usize,
    pub line: usize,
}

impl Token {
    pub fn new(kind: TokenKind) -> Self {
        Token { kind, num_val: 0, str_val: String::new(), col: 0, line: 0 }
    }
}

pub struct Lexer {
    src: Vec<u8>,
    pos: usize,
    pub current: Token,
    pub line: usize,
    pub col: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        let mut lexer = Lexer {
            src: source.as_bytes().to_vec(),
            pos: 0,
            current: Token::new(TokenKind::Eof),
            line: 1,
            col: 1,
        };
        lexer.next();
        lexer
    }

    fn eof(&self) -> bool { self.pos >= self.src.len() }
    fn ch(&self) -> u8 { if self.eof() { 0 } else { self.src[self.pos] } }
    fn peek_ch(&self) -> u8 { if self.pos + 1 >= self.src.len() { 0 } else { self.src[self.pos + 1] } }
    fn advance(&mut self) {
        if !self.eof() {
            if self.src[self.pos] == b'\n' { self.line += 1; self.col = 1; }
            else { self.col += 1; }
            self.pos += 1;
        }
    }

    fn skip_ws(&mut self) {
        while !self.eof() && (self.ch() == b' ' || self.ch() == b'\t') {
            self.advance();
        }
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        Token { kind, num_val: 0, str_val: String::new(), col: self.col, line: self.line }
    }

    fn read_num(&mut self) -> i32 {
        let mut n: i32 = 0;
        while !self.eof() && self.ch() >= b'0' && self.ch() <= b'9' {
            n = n.wrapping_mul(10).wrapping_add((self.ch() - b'0') as i32);
            self.advance();
        }
        n
    }

    fn read_hex(&mut self) -> i32 {
        self.advance(); // skip '0'
        self.advance(); // skip 'x'
        let mut n: i32 = 0;
        while !self.eof() {
            let c = self.ch();
            let digit = match c {
                b'0'..=b'9' => (c - b'0') as i32,
                b'a'..=b'f' => (c - b'a' + 10) as i32,
                b'A'..=b'F' => (c - b'A' + 10) as i32,
                _ => break,
            };
            n = n.wrapping_mul(16).wrapping_add(digit);
            self.advance();
        }
        n
    }

    fn read_ident(&mut self) -> String {
        let mut name = String::new();
        while !self.eof() && (self.ch() == b'_' || self.ch().is_ascii_alphanumeric() || self.ch() == b'!') {
            name.push(self.ch() as char);
            self.advance();
        }
        name
    }

    fn read_string(&mut self) -> String {
        self.advance(); // skip opening "
        let mut s = String::new();
        while !self.eof() && self.ch() != b'"' {
            if self.ch() == b'\\' {
                self.advance();
                match self.ch() {
                    b'n' => s.push('\n'),
                    b't' => s.push('\t'),
                    b'\\' => s.push('\\'),
                    b'"' => s.push('"'),
                    b'$' => s.push('$'),
                    _ => { s.push('\\'); s.push(self.ch() as char); }
                }
            } else {
                s.push(self.ch() as char);
            }
            self.advance();
        }
        if !self.eof() { self.advance(); } // skip closing "
        s
    }

    fn check_keyword(name: &str) -> TokenKind {
        match name {
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "elseif" => TokenKind::ElseIf,
            "end" => TokenKind::End,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "function" => TokenKind::Function,
            "return" => TokenKind::Return,
            "println" => TokenKind::Println,
            "print" => TokenKind::Print,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "local" => TokenKind::Local,
            "include" => TokenKind::Include,
            _ => TokenKind::Ident,
        }
    }

    pub fn next(&mut self) {
        self.skip_ws();
        if self.eof() { self.current = self.make_token(TokenKind::Eof); return; }

        let c = self.ch();
        let col = self.col;
        let line = self.line;

        // Newline
        if c == b'\n' || c == b'\r' {
            self.advance();
            if c == b'\r' && !self.eof() && self.ch() == b'\n' { self.advance(); }
            self.current = Token { kind: TokenKind::Newline, num_val: 0, str_val: String::new(), col, line };
            return;
        }

        // Semicolon = newline
        if c == b';' {
            self.advance();
            self.current = Token { kind: TokenKind::Newline, num_val: 0, str_val: String::new(), col, line };
            return;
        }

        // Comment # (single line)
        if c == b'#' {
            if self.peek_ch() == b'=' {
                // Multi-line comment #= ... =#
                self.advance(); self.advance();
                loop {
                    if self.eof() { break; }
                    if self.ch() == b'=' && self.peek_ch() == b'#' {
                        self.advance(); self.advance();
                        break;
                    }
                    self.advance();
                }
                self.next(); return;
            }
            while !self.eof() && self.ch() != b'\n' { self.advance(); }
            self.next(); return;
        }

        // String literal
        if c == b'"' {
            let s = self.read_string();
            self.current = Token { kind: TokenKind::StringLit, num_val: 0, str_val: s, col, line };
            return;
        }

        // Number (hex or decimal)
        if c >= b'0' && c <= b'9' {
            if c == b'0' && self.peek_ch() == b'x' {
                let n = self.read_hex();
                self.current = Token { kind: TokenKind::HexNum, num_val: n, str_val: String::new(), col, line };
            } else {
                let n = self.read_num();
                self.current = Token { kind: TokenKind::Num, num_val: n, str_val: String::new(), col, line };
            }
            return;
        }

        // Identifier or keyword
        if c == b'_' || c.is_ascii_alphabetic() {
            let name = self.read_ident();
            let kind = Self::check_keyword(&name);
            self.current = Token { kind, num_val: 0, str_val: name, col, line };
            return;
        }

        // Operators
        match c {
            b'+' => { self.advance(); if !self.eof() && self.ch() == b'=' { self.advance(); self.current = self.tok(TokenKind::PlusEq, col, line); } else { self.current = self.tok(TokenKind::Plus, col, line); } }
            b'-' => {
                self.advance();
                if !self.eof() && self.ch() == b'=' { self.advance(); self.current = self.tok(TokenKind::MinusEq, col, line); }
                else if !self.eof() && self.ch() == b'>' { self.advance(); self.current = self.tok(TokenKind::Arrow, col, line); }
                else { self.current = self.tok(TokenKind::Minus, col, line); }
            }
            b'*' => { self.advance(); if !self.eof() && self.ch() == b'=' { self.advance(); self.current = self.tok(TokenKind::StarEq, col, line); } else { self.current = self.tok(TokenKind::Star, col, line); } }
            b'/' => { self.advance(); if !self.eof() && self.ch() == b'=' { self.advance(); self.current = self.tok(TokenKind::SlashEq, col, line); } else { self.current = self.tok(TokenKind::Slash, col, line); } }
            b'%' => { self.advance(); if !self.eof() && self.ch() == b'=' { self.advance(); self.current = self.tok(TokenKind::PercentEq, col, line); } else { self.current = self.tok(TokenKind::Percent, col, line); } }
            b'^' => { self.advance(); self.current = self.tok(TokenKind::Caret, col, line); }
            b'(' => { self.advance(); self.current = self.tok(TokenKind::LParen, col, line); }
            b')' => { self.advance(); self.current = self.tok(TokenKind::RParen, col, line); }
            b'[' => { self.advance(); self.current = self.tok(TokenKind::LBracket, col, line); }
            b']' => { self.advance(); self.current = self.tok(TokenKind::RBracket, col, line); }
            b',' => { self.advance(); self.current = self.tok(TokenKind::Comma, col, line); }
            b':' => { self.advance(); self.current = self.tok(TokenKind::Colon, col, line); }
            b'?' => { self.advance(); self.current = self.tok(TokenKind::Question, col, line); }
            b'$' => { self.advance(); self.current = self.tok(TokenKind::Dollar, col, line); }
            b'~' => { self.advance(); self.current = self.tok(TokenKind::Tilde, col, line); }
            b'!' => {
                self.advance();
                if !self.eof() && self.ch() == b'=' { self.advance(); self.current = self.tok(TokenKind::Neq, col, line); }
                else { self.current = self.tok(TokenKind::Not, col, line); }
            }
            b'=' => {
                self.advance();
                if !self.eof() && self.ch() == b'=' { self.advance(); self.current = self.tok(TokenKind::Eq, col, line); }
                else { self.current = self.tok(TokenKind::Assign, col, line); }
            }
            b'>' => {
                self.advance();
                if !self.eof() && self.ch() == b'=' { self.advance(); self.current = self.tok(TokenKind::Gte, col, line); }
                else if !self.eof() && self.ch() == b'>' { self.advance(); self.current = self.tok(TokenKind::Shr, col, line); }
                else { self.current = self.tok(TokenKind::Gt, col, line); }
            }
            b'<' => {
                self.advance();
                if !self.eof() && self.ch() == b'=' { self.advance(); self.current = self.tok(TokenKind::Lte, col, line); }
                else if !self.eof() && self.ch() == b'<' { self.advance(); self.current = self.tok(TokenKind::Shl, col, line); }
                else { self.current = self.tok(TokenKind::Lt, col, line); }
            }
            b'&' => {
                self.advance();
                if !self.eof() && self.ch() == b'&' { self.advance(); self.current = self.tok(TokenKind::And, col, line); }
                else { self.current = self.tok(TokenKind::Ampersand, col, line); }
            }
            b'|' => {
                self.advance();
                if !self.eof() && self.ch() == b'|' { self.advance(); self.current = self.tok(TokenKind::Or, col, line); }
                else { self.current = self.tok(TokenKind::Pipe, col, line); }
            }
            _ => { self.advance(); self.next(); }
        }
    }

    fn tok(&self, kind: TokenKind, col: usize, line: usize) -> Token {
        Token { kind, num_val: 0, str_val: String::new(), col, line }
    }

    pub fn kind(&self) -> TokenKind { self.current.kind }
}
