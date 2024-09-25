#[derive(Debug)]
pub enum NumberType {
    Float,
    Hex,
    Bin,
    Octal,
    Seq,
}

#[derive(Debug)]
pub enum TokenType {
    Word,
    Number(NumberType),
    String,
    Symbol,
}

#[derive(Debug)]
pub struct Loc(pub usize, pub usize);

#[derive(Debug)]
pub struct Token {
    pub r#type: TokenType,
    pub value: Box<String>,
    pub loc: Loc,
}

pub struct Tokenizer {
    pub lines: Vec<Vec<char>>,
    ln: usize,
    col: usize,
}

pub struct TokenizerBuilder {
    string_del: char,
    char_token_exist: bool,
    char_del: char,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutOfBound {
    Empty,
    Out,
    Within,
}

impl Tokenizer {
    pub fn new<T>(input: T) -> Self
    where
        T: ToString,
    {
        let input = input.to_string();
        Self {
            lines: input.lines().map(|line| line.chars().collect()).collect(),
            ln: 0,
            col: 0,
        }
    }

    fn is_out_of_bound(&self) -> OutOfBound {
        if self.ln >= self.lines.len() {
            return OutOfBound::Out;
        } else if !self.lines[self.ln].is_empty() {
            if self.col >= self.lines[self.ln].len() {
                return OutOfBound::Out;
            }
        } else {
            return OutOfBound::Empty;
        }

        OutOfBound::Within

        // return self.ln >= self.lines.len()
        //     || if !self.lines[self.ln].is_empty() {
        //         self.col >= self.lines[self.ln].len()
        //     } else {
        //         // FIXME: fix this by ignoring empty lines
        //         false
        //     };
    }

    #[track_caller]
    fn consume(&mut self, len: usize) {
        let mut len = len;
        let ln_max = self.lines.len();

        while len > 0 {
            if self.ln + 1 > ln_max {
                // At this point, calling consume will have no effect,
                // The cursor is already out of bound
                // NOTE: don't put `unreachable!()` to avoid panic
                if self.is_out_of_bound() == OutOfBound::Out {
                    let caller_location = std::panic::Location::caller();
                    eprintln!(
                        "WARNING at {} The consume function is called but we are out of characters",
                        caller_location
                    );
                }

                self.ln += 1;
                break;
            } else {
                let col_max = self.lines[self.ln].len();
                if self.col + 1 >= col_max {
                    self.ln += 1;
                    self.col = 0;
                } else {
                    self.col += 1;
                }
            }

            len -= 1;
        }
    }

    fn get_lc(&self) -> Option<&char> {
        if self.is_out_of_bound() == OutOfBound::Within {
            Some(&self.lines[self.ln][self.col])
        } else {
            None
        }
    }

    pub fn parse_word(&mut self) -> Token {
        let mut word = String::new();
        let start_ln = self.ln;
        let start_col = self.col;
        while let Some(c) = self.get_lc() {
            if *c != ' ' {
                word.push(*c);
            } else {
                break;
            }
            self.consume(1);
        }

        Token {
            r#type: TokenType::Word,
            value: Box::new(word),
            loc: Loc(start_ln, start_col),
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = vec![];
        while self.is_out_of_bound() != OutOfBound::Out {
            if self.is_out_of_bound() == OutOfBound::Empty {
                self.consume(1);
            } else {
                match self.get_lc().unwrap() {
                    ' ' => {
                        self.consume(1);
                    }
                    _ => tokens.push(self.parse_word()),
                }
            }
        }

        tokens
    }
}
