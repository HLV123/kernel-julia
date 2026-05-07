// ============================================================
// lexer.rs -- Bộ phân tích từ tố (Lexer)
// Chuyển mã nguồn Julia thành luồng token
// (chuyển từ 12-lexer.fs sang Rust)
// ============================================================

use alloc::string::String;

/// Loại token
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    Eof,       // Hết mã nguồn
    Num,       // Số nguyên
    Ident,     // Tên biến / hàm
    Assign,    // Phép gán =
    Plus,      // Phép cộng +
    Minus,     // Phép trừ -
    Star,      // Phép nhân *
    LParen,    // Ngoặc mở (
    RParen,    // Ngoặc đóng )
    Comma,     // Dấu phẩy ,
    Newline,   // Xuống dòng hoặc ;
    Eq,        // So sánh ==
    Neq,       // Khác !=
    Gt,        // Lớn hơn >
    Lt,        // Nhỏ hơn <
    Gte,       // Lớn bằng >=
    Lte,       // Nhỏ bằng <=
    If,        // Từ khoá if
    Else,      // Từ khoá else
    ElseIf,    // Từ khoá elseif
    End,       // Từ khoá end
    While,     // Từ khoá while
    Function,  // Từ khoá function
    Return,    // Từ khoá return
    Println,   // Từ khoá println
}

/// Token — đơn vị từ tố
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub num_val: i32,          // Giá trị số (cho Num)
    pub ident: String,         // Tên biến/hàm (cho Ident)
}

impl Token {
    pub fn new(kind: TokenKind) -> Self {
        Token { kind, num_val: 0, ident: String::new() }
    }
}

/// Bộ phân tích từ tố (Lexer)
pub struct Lexer {
    /// Mã nguồn dưới dạng bytes
    src: alloc::vec::Vec<u8>,
    /// Vị trí hiện tại trong mã nguồn
    pos: usize,
    /// Token hiện tại
    pub current: Token,
}

impl Lexer {
    /// Tạo lexer mới từ chuỗi mã nguồn
    pub fn new(source: &str) -> Self {
        let mut lexer = Lexer {
            src: source.as_bytes().to_vec(),
            pos: 0,
            current: Token::new(TokenKind::Eof),
        };
        lexer.next(); // Nạp token đầu tiên
        lexer
    }

    // --- Hàm kiểm tra ký tự ---

    fn eof(&self) -> bool {
        self.pos >= self.src.len()
    }

    fn ch(&self) -> u8 {
        if self.eof() { 0 } else { self.src[self.pos] }
    }

    fn advance(&mut self) {
        if self.pos < self.src.len() {
            self.pos += 1;
        }
    }

    fn is_ws(c: u8) -> bool {
        c == b' ' || c == b'\t'
    }

    fn is_nl(c: u8) -> bool {
        c == b'\n' || c == b'\r'
    }

    fn is_digit(c: u8) -> bool {
        c >= b'0' && c <= b'9'
    }

    fn is_alpha(c: u8) -> bool {
        c == b'_' || (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z')
    }

    fn is_alnum(c: u8) -> bool {
        Self::is_alpha(c) || Self::is_digit(c)
    }

    // --- Bỏ qua khoảng trắng (không bỏ dòng mới) ---
    fn skip_ws(&mut self) {
        while !self.eof() && Self::is_ws(self.ch()) {
            self.advance();
        }
    }

    // --- Đọc số nguyên ---
    fn read_num(&mut self) -> i32 {
        let mut n: i32 = 0;
        while !self.eof() && Self::is_digit(self.ch()) {
            n = n.wrapping_mul(10).wrapping_add((self.ch() - b'0') as i32);
            self.advance();
        }
        n
    }

    // --- Đọc tên biến / hàm ---
    fn read_ident(&mut self) -> String {
        let mut name = String::new();
        while !self.eof() && Self::is_alnum(self.ch()) {
            name.push(self.ch() as char);
            self.advance();
        }
        name
    }

    // --- Kiểm tra từ khoá ---
    fn check_keyword(name: &str) -> TokenKind {
        match name {
            "if"       => TokenKind::If,
            "else"     => TokenKind::Else,
            "elseif"   => TokenKind::ElseIf,
            "end"      => TokenKind::End,
            "while"    => TokenKind::While,
            "function" => TokenKind::Function,
            "return"   => TokenKind::Return,
            "println"  => TokenKind::Println,
            _          => TokenKind::Ident,
        }
    }

    /// Đọc token tiếp theo (tương đương jl-next trong Forth)
    pub fn next(&mut self) {
        self.skip_ws();

        if self.eof() {
            self.current = Token::new(TokenKind::Eof);
            return;
        }

        let c = self.ch();

        // Dòng mới
        if Self::is_nl(c) {
            self.advance();
            self.current = Token::new(TokenKind::Newline);
            return;
        }

        // Dấu chấm phẩy = dòng mới
        if c == b';' {
            self.advance();
            self.current = Token::new(TokenKind::Newline);
            return;
        }

        // Comment (#)
        if c == b'#' {
            while !self.eof() && !Self::is_nl(self.ch()) {
                self.advance();
            }
            if !self.eof() { self.advance(); }
            self.current = Token::new(TokenKind::Newline);
            return;
        }

        // Số nguyên
        if Self::is_digit(c) {
            let n = self.read_num();
            self.current = Token { kind: TokenKind::Num, num_val: n, ident: String::new() };
            return;
        }

        // Tên biến / từ khoá
        if Self::is_alpha(c) {
            let name = self.read_ident();
            let kind = Self::check_keyword(&name);
            self.current = Token { kind, num_val: 0, ident: name };
            return;
        }

        // Toán tử
        match c {
            b'+' => { self.advance(); self.current = Token::new(TokenKind::Plus); }
            b'-' => { self.advance(); self.current = Token::new(TokenKind::Minus); }
            b'*' => { self.advance(); self.current = Token::new(TokenKind::Star); }
            b'(' => { self.advance(); self.current = Token::new(TokenKind::LParen); }
            b')' => { self.advance(); self.current = Token::new(TokenKind::RParen); }
            b',' => { self.advance(); self.current = Token::new(TokenKind::Comma); }
            b'=' => {
                self.advance();
                if !self.eof() && self.ch() == b'=' {
                    self.advance();
                    self.current = Token::new(TokenKind::Eq);
                } else {
                    self.current = Token::new(TokenKind::Assign);
                }
            }
            b'!' => {
                self.advance();
                if !self.eof() && self.ch() == b'=' {
                    self.advance();
                    self.current = Token::new(TokenKind::Neq);
                } else {
                    // Ký tự '!' không hợp lệ một mình, bỏ qua
                    self.current = Token::new(TokenKind::Eof);
                }
            }
            b'>' => {
                self.advance();
                if !self.eof() && self.ch() == b'=' {
                    self.advance();
                    self.current = Token::new(TokenKind::Gte);
                } else {
                    self.current = Token::new(TokenKind::Gt);
                }
            }
            b'<' => {
                self.advance();
                if !self.eof() && self.ch() == b'=' {
                    self.advance();
                    self.current = Token::new(TokenKind::Lte);
                } else {
                    self.current = Token::new(TokenKind::Lt);
                }
            }
            _ => {
                // Ký tự lạ — bỏ qua
                self.advance();
                self.next();
            }
        }
    }

    /// Kiểm tra token hiện tại
    pub fn kind(&self) -> TokenKind {
        self.current.kind
    }

    /// Lấy giá trị số của token hiện tại
    pub fn num_val(&self) -> i32 {
        self.current.num_val
    }

    /// Lấy tên identifier của token hiện tại
    pub fn ident(&self) -> &str {
        &self.current.ident
    }
}
