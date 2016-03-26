use itertools::Itertools;
use std::fmt;
use std::str::Chars;

use context::EvalContext;
use symbol_table::Symbol;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Pos {
    column: usize,
    line: usize,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Span {
    filename: Symbol,
    start: Pos,
    end: Pos,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Spanned<T> {
    pub val: T,
    pub span: Span,
}

pub type Token = Spanned<TokenKind>;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Unknown,

    // Basic
    Id(Symbol),
    Int(i64),
    Float(f64),
    Path(Symbol),

    // String-related
    Uri(String),
    StrPart(String),
    IndentStrPart(String),
    Quote,          // "
    IndentQuote,    // ''
    DollarBrace,    // ${

    // Operators
    Mult,       // *
    Minus,      // -
    Plus,       // +
    Divide,     // /
    Less,       // <
    Greater,    // >
    LessEq,     // <=
    GreaterEq,  // >=
    Assign,     // =
    Equals,     // ==
    NotEquals,  // !=
    And,        // &&
    Or,         // ||
    Implies,    // ->
    Not,        // !
    Update,     // //
    Concat,     // ++

    // Other syntax
    At,         // @
    Comma,      // ,
    Dot,        // .
    Ellipsis,   // ...
    Question,   // ?
    Colon,      // :
    Semicolon,  // ;

    // Delimiters
    ParenL,     // (
    ParenR,     // )
    BracketL,   // [
    BracketR,   // ]
    BraceL,     // {
    BraceR,     // }
}

pub struct Lexer<'ctx, 'src> {
    ectx: &'ctx EvalContext,
    source: &'src str,
    chars: CharsPos<'src>,
    filename: Symbol,
}

impl<'ctx, 'src> Lexer<'ctx, 'src> {
    pub fn new(ectx: &'ctx EvalContext, filename: &str, source: &'src str) -> Self {
        Lexer {
            ectx: ectx,
            source: source,
            chars: CharsPos::new(source.chars()),
            filename: ectx.intern(filename),
        }
    }

    fn lex_int(&mut self) -> Token {
        let start = self.pos();
        let chars = self.chars.as_str();
        let num_digits = self.chars.take_while_ref(|c| c.is_digit(10)).count();
        let digits = &chars[..num_digits];

        // TODO(tsion): Detect and diagnose integer overflow.
        self.spanned(start, self.pos(), TokenKind::Int(digits.parse::<i64>().unwrap()))
    }

    fn peek(&self) -> Option<char> {
        self.chars.clone().next()
    }

    fn pos(&self) -> Pos {
        self.chars.pos
    }

    fn spanned<T>(&self, start: Pos, end: Pos, val: T) -> Spanned<T> {
        Spanned {
            val: val,
            span: Span { filename: self.filename, start: start, end: end },
        }
    }
}

impl<'ctx, 'src> Iterator for Lexer<'ctx, 'src> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        match self.peek() {
            Some(c) if c.is_digit(10) => Some(self.lex_int()),
            Some(c) => panic!("unhandled char: {}", c),
            None => None,
        }
    }
}

/// An iterator wrapping a `std::str::Chars` iterator which also keeps track of the current line
/// and column position.
#[derive(Clone)]
struct CharsPos<'a> {
    chars: Chars<'a>,
    pos: Pos,
}

impl<'a> CharsPos<'a> {
    fn new(chars: Chars<'a>) -> Self {
        CharsPos { chars: chars, pos: Pos { line: 1, column: 1 } }
    }

    fn as_str(&self) -> &'a str {
        self.chars.as_str()
    }
}

impl<'a> Iterator for CharsPos<'a> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        let opt_c = self.chars.next();
        match opt_c {
            Some('\n') => { self.pos.line += 1; self.pos.column = 1; }
            Some(_) => { self.pos.column += 1; }
            None => {}
        }
        opt_c
    }
}

pub fn lex(ectx: &EvalContext, filename: &str, source: &str) -> Vec<Token> {
    Lexer::new(ectx, filename, source).collect()
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[cfg(test)]
mod test {
    use context::EvalContext;
    use parse::{Lexer, TokenKind};

    macro_rules! assert_lex {
        ($src:expr => [ $($span:expr => $token:expr),* ]) => ({
            let expected = [ $(($token, String::from($span))),* ];
            assert_eq!(lex($src), expected);
        })
    }

    fn lex(src: &str) -> Vec<(TokenKind, String)> {
        Lexer::new(&EvalContext::new(), "<test>", src)
            .map(|t| (t.val, format!("{}-{}", t.span.start, t.span.end)))
            .collect()
    }

    #[test]
    fn test_lex() {
        use parse::TokenKind::*;
        assert_lex!("" => []);
        assert_lex!("0" => ["1:1-1:2" => Int(0)]);
    }
}
