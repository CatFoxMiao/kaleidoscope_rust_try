pub enum Token {
    Eof,
    Def,
    Extern,
    Identifier,
    Number,
    Char(char),
    Comment,
}

enum CharState {
    NotInitailized,
    Char(char),
    Eof
}

use std::io::{self, Read};
// 获取当前字符
// struct Lexer {
//     source: Stdin,
//     last_char: Option<char>,
//     identifier_str: String,
//     num_val: Option<f64>,
//     // line:u32,
//     // column:u32
// }
pub struct Lexer<R: Read> {
    source: R, // 使用泛型 R 替代固定的 Stdin
    last_char: Option<char>,
    identifier_str: String,
    num_val: Option<f64>,
}

impl<R: Read> Lexer<R> {
    pub fn new(source: R) -> io::Result<Self> {
        // let mut buf = [0u8; 1];
        // let last_char = match source.read_exact(&mut buf) {
        //     Ok(_) => Some(buf[0] as char),                              // 成功读取
        //     Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => None, // EOF
        //     Err(e) => return Err(e),                                    // 其他错误向上传递
        // };
        // Ok(Lexer {
        //     source: source,
        //     last_char: last_char,
        //     identifier_str: String::new(),
        //     num_val: None,
        // })
        Ok(Lexer {
            source: source,
            last_char: Some(' '), // 初始化为空格以跳过前导空格
            identifier_str: String::new(),
            num_val: None,
        })
    }

    pub fn get_char(&mut self) {
        let mut buf = [0u8; 1];
        match self.source.read_exact(&mut buf) {
            Ok(_) => {
                self.last_char = Some(buf[0] as char);
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                self.last_char = None;
            }
            Err(e) => {
                eprintln!("文件读取失败: {}", e);
            }
        }
    }

    // get the token
    pub fn get_token(&mut self) -> Token {
        // Skip any whitspace

        while self.last_char == Some(' ') {
            // println!("self.last_char:{}", self.last_char.unwrap());
            self.get_char();
        }
        if self.last_char == None {
            return Token::Eof;
        }
        // println!("break while");
        // println!("self.last_char:{}", self.last_char.unwrap());
        // identifying idenfifiers and keywords such as "def" and "extern"

        if self.last_char.unwrap().is_alphabetic() {
            self.identifier_str.clear();
            self.identifier_str.push(self.last_char.unwrap());
            loop {
                self.get_char();
                // after get a variable,
                if self.last_char.map(|c| !c.is_alphanumeric()).unwrap_or(true) {
                    break;
                }
                self.identifier_str.push(self.last_char.unwrap());
            }

            if self.identifier_str.as_str() == "def" {
                return Token::Def;
            };
            if self.identifier_str.as_str() == "extern" {
                return Token::Extern;
            };
            println!("self.identifier:{}", self.identifier_str);
            return Token::Identifier;
        }

        // Number:[0-9.]+
        if self.last_char.unwrap().is_numeric() || self.last_char.unwrap() == '.' {
            let mut number_str = String::new();
            loop {
                number_str.push(self.last_char.unwrap());
                self.get_char();

                if self
                    .last_char
                    .map(|c| !c.is_numeric() && c != '.')
                    .unwrap_or(true)
                {
                    self.num_val = number_str.parse::<f64>().ok();
                    return Token::Number;
                }
            }
        }

        if self.last_char.unwrap() == '#' {
            loop {
                if !(self.last_char.unwrap() != '\0'
                    && self.last_char.unwrap() != '\n'
                    && self.last_char.unwrap() != '\r')
                {
                    break;
                };
            }
            return Token::Comment;
        }

        // Otherwise just return the character as its ascii value.
        let this_char: char = self.last_char.unwrap();
        self.get_char();
        Token::Char(this_char)
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
        assert!(matches!(lexer1.last_char.unwrap(), 'a'));
        lexer1.get_char();
        assert!(matches!(lexer1.last_char.unwrap(), 'b'));
        lexer1.get_char();
        assert!(matches!(lexer1.last_char.unwrap(), 'c'));
        lexer1.get_char();
        assert!(matches!(lexer1.last_char, None));
    }

    #[test]
    fn test_skip_spaces() {
        let mut lexer1 = create_lexer("   a");
        assert!(matches!(lexer1.get_token(), Token::Identifier));
        assert!(matches!(lexer1.get_token(), Token::Eof));
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
}
