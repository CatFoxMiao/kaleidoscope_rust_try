#[derive(Copy, Clone)]
pub enum Token {
    None,
    Eof,
    Def,
    Extern,
    Identifier,
    Number,
    Char(char),
    Comment,
}

#[derive(PartialEq)]
pub enum CharState {
    NotInitailized,
    Char(char),
    Eof,
}

impl CharState {
    pub fn is_alphabetic(&self) -> bool {
        match self {
            CharState::Char(c) => c.is_alphabetic(),
            _ => false,
        }
    }
}

use std::{
    char, io::{self, Read}, ptr::read_unaligned, rc::Rc
};

pub struct Lexer<R: Read> {
    source: R, // 使用泛型 R 替代固定的 Stdin
    last_char: CharState,
    identifier_str: String,
    num_val: Option<f64>,
    cur_tok:Token
}

impl<R: Read> Lexer<R> {
    pub fn new(source: R) -> io::Result<Self> {
        Ok(Lexer {
            source: source,
            last_char: CharState::NotInitailized, // 初始化为空格以跳过前导空格
            identifier_str: String::new(),
            num_val: None,
            cur_tok:Token::None
        })
    }

    pub fn get_char(&mut self) {
        let mut buf = [0u8; 1];
        match self.source.read_exact(&mut buf) {
            Ok(_) => {
                self.last_char = CharState::Char(buf[0] as char);
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                self.last_char = CharState::Eof;
            }
            Err(e) => {
                eprintln!("文件读取失败: {}", e);
            }
        }
    }

    pub fn get_token(&mut self) -> Token {
        // 跳过空格
        while self.last_char == CharState::Char(' ') || self.last_char == CharState::NotInitailized
        {
            self.get_char();
        }

        match self.last_char {
            // determine whether is eof
            CharState::Eof => return Token::Eof,

            // determin whether is identifier eof extern
            CharState::Char(c) if c.is_alphabetic() => {
                self.identifier_str.clear();
                self.identifier_str.push(c);
                loop {
                    self.get_char();
                    match self.last_char {
                        CharState::Char(this_c) if this_c.is_alphanumeric() => {
                            self.identifier_str.push(this_c);
                        }
                        _ => break,
                    }
                }

                match self.identifier_str.as_str() {
                    "def" => Token::Def,
                    "extern" => Token::Extern,
                    _ => Token::Identifier,
                }
            }

            CharState::Char(c) if c.is_numeric() || c == '.' => {
                let mut number_str = String::new();
                loop {
                    if let CharState::Char(num_c) = self.last_char {
                        number_str.push(num_c);
                        self.get_char();

                        match self.last_char {
                            CharState::Char(next_c) if next_c.is_numeric() || next_c == '.' => {
                                continue;
                            }
                            _ => break,
                        }
                    } else {
                        break;
                    }
                }
                self.num_val = number_str.parse::<f64>().ok();
                Token::Number
            }

            CharState::Char(c) => {
                self.get_char();
                Token::Char(c)
            }
            CharState::NotInitailized => unreachable!(),
        }
    }

    pub fn get_next_token(&mut self) -> Token{
        self.cur_tok = self.get_token();
        return  self.cur_tok;
    }
}

use std::any::Any;
use std::fmt::Debug;

// Abstract Syntax Tree(aka Parse Tree)
pub trait ExprAST: Any + Debug {
    fn as_any(&self) -> &dyn Any;
}

// macro automatic implement ExprAST for Structs
macro_rules! impl_expr_ast {
    ($($struct_name:ident),*) => {
        $(
            impl ExprAST for $struct_name{
                fn as_any(&self)-> &dyn Any{
                    self
                }
            }
        )*
    };
}

// NumberExprAST - Expression struct for numeric literals like "1.0"
#[derive(Debug)]
pub struct NumberExprAST {
    val: f64,
}
impl NumberExprAST {
    pub fn new(val: f64) -> Self {
        NumberExprAST { val: val }
    }
}
#[derive(Debug)]
pub struct VariableExprAST {
    name: String,
}
impl VariableExprAST {
    pub fn new(name: String) -> Self {
        VariableExprAST { name: name }
    }
}

#[derive(Debug)]
pub struct BinaryExprAST {
    op: char,
    lhs: Rc<dyn ExprAST>,
    rhs: Rc<dyn ExprAST>,
}
impl BinaryExprAST {
    pub fn new(op: char, lhs: Rc<dyn ExprAST>, rhs: Rc<dyn ExprAST>) -> BinaryExprAST {
        BinaryExprAST {
            op: op,
            lhs: lhs,
            rhs: rhs,
        }
    }
}
#[derive(Debug)]
pub struct CallExprAST {
    callee: String,
    args: Vec<Rc<dyn ExprAST>>,
}
impl CallExprAST {
    pub fn new(callee: String, args: Vec<Rc<dyn ExprAST>>) -> Self {
        CallExprAST {
            callee: callee,
            args: args,
        }
    }
}
#[derive(Debug)]
pub struct PrototypeAST {
    name: String,
    args: Vec<String>,
}
impl PrototypeAST {
    pub fn new(name: String, args: Vec<String>) -> PrototypeAST {
        PrototypeAST {
            name: name,
            args: args,
        }
    }
}
#[derive(Debug)]
pub struct FunctionAST {
    proto: Rc<PrototypeAST>,
    body: Rc<dyn ExprAST>,
}
impl FunctionAST {
    pub fn new(proto: Rc<PrototypeAST>, body: Rc<dyn ExprAST>) -> Self {
        FunctionAST {
            proto: proto,
            body: body,
        }
    }
}

impl_expr_ast!(
    NumberExprAST,
    VariableExprAST,
    BinaryExprAST,
    CallExprAST,
    PrototypeAST,
    FunctionAST
);

#[cfg(test)]
mod test_ast {
    use super::*;

    #[test]
    fn test_one() {
        let x: Rc<dyn ExprAST> = Rc::new(VariableExprAST::new("x".into()));
        let y: Rc<dyn ExprAST> = Rc::new(VariableExprAST::new("y".into()));
        let result = Rc::new(BinaryExprAST::new('+', x.clone(), y.clone()));
        

    
    }
}

#[cfg(test)]
mod test_lexer {
    use super::*;
    // 替代stdin 模拟输入结构体
    #[cfg(test)]
    struct MockReader {
        data: Vec<u8>,
        position: usize,
    }

    #[cfg(test)]
    impl Read for MockReader {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.position >= self.data.len() {
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
            }
            let len = buf.len().min(self.data.len() - self.position);
            buf[..len].copy_from_slice(&self.data[self.position..self.position + len]);
            self.position += len;
            Ok(len)
        }
    }

    #[cfg(test)]
    fn create_lexer(input: &str) -> Lexer<MockReader> {
        let source_mock_reader = MockReader {
            data: input.as_bytes().to_vec(),
            position: 0,
        };

        Lexer::new(source_mock_reader).unwrap()
    }
    #[test]
    fn test_mock() {
        let mut lexer1 = create_lexer("abc");
        lexer1.get_char();
        assert!(matches!(lexer1.last_char, CharState::Char('a')));
        lexer1.get_char();
        assert!(matches!(lexer1.last_char, CharState::Char('b')));
        lexer1.get_char();
        assert!(matches!(lexer1.last_char, CharState::Char('c')));
        lexer1.get_char();
        assert!(matches!(lexer1.last_char, CharState::Eof));
    }

    #[test]
    fn test_skip_spaces() {
        let mut lexer1 = create_lexer("   a");
        assert!(matches!(lexer1.get_token(), Token::Identifier));
        //assert!(matches!(lexer1.get_token(), Token::Eof));
        // assert_eq!(lexer.last_char, Some('a')); // 正确停在第一个非空格字符
    }
    #[test]
    fn test_eof() {
        let mut lexer1 = create_lexer("");
        assert!(matches!(lexer1.get_token(), Token::Eof));
        let mut lexer2 = create_lexer("    ");
        assert!(matches!(lexer2.get_token(), Token::Eof));
    }

    #[test]
    fn test_def() {
        let mut lexer1 = create_lexer("def");
        assert!(matches!(lexer1.get_token(), Token::Def));
        let mut lexer2 = create_lexer("   def  ");
        assert!(matches!(lexer2.get_token(), Token::Def));
        assert!(matches!(lexer1.get_token(), Token::Eof));
    }

    #[test]
    fn test_extern() {
        let mut lexer1 = create_lexer("extern");
        assert!(matches!(lexer1.get_token(), Token::Extern));
        let mut lexer2 = create_lexer("   extern  ");
        assert!(matches!(lexer2.get_token(), Token::Extern));
        assert!(matches!(lexer1.get_token(), Token::Eof));
    }
    #[test]
    fn test_identifier() {
        let mut lexer1 = create_lexer("abc");
        //assert!(matches!(lexer1.identifier_str.as_str(), "abc"));
        //assert_eq!(lexer1.identifier_str.as_str(), "abc");
        assert!(matches!(lexer1.get_token(), Token::Identifier));
        assert!(matches!(lexer1.get_token(), Token::Eof));
    }

    #[test]
    fn test_number() {
        let mut lexer1 = create_lexer("1.234");
        assert!(matches!(lexer1.get_token(), Token::Number));
        assert!(matches!(lexer1.num_val, Some::<f64>(1.234)));
        assert!(matches!(lexer1.get_token(), Token::Eof));
        let mut lexer2 = create_lexer(".234");
        assert!(matches!(lexer2.get_token(), Token::Number));
        assert!(matches!(lexer2.num_val, Some::<f64>(0.234)));
        let mut lexer2 = create_lexer("       .234");
        assert!(matches!(lexer2.get_token(), Token::Number));
        assert!(matches!(lexer2.num_val, Some::<f64>(0.234)));
    }
    // let mut lexer2 = create_lexer("12.3");
    // assert!(matches!(lexer2.get_token(),Token::Number));

    #[test]
    fn test_char() {
        let mut lexer1 = create_lexer("a+b");
        assert!(matches!(lexer1.get_token(), Token::Identifier));
        assert!(matches!(lexer1.get_token(), Token::Char('+')));
        assert!(matches!(lexer1.get_token(), Token::Identifier));
    }
}
