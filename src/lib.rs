#[derive(Debug, PartialEq, Eq)]
pub enum NumberType {
    Float,
    Hex,
    Binary,
    Octal,
    Seq,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Word,
    Number(NumberType),
    String,
    Char,
    Symbol,
}

#[derive(Debug)]
pub struct Loc(pub usize, pub usize);

impl std::fmt::Display for Loc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.0 + 1, self.1 + 1)
    }
}

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
    config: TokenizerConfig,
}

#[derive(Clone)]
pub enum Choice<T> {
    Yes(T),
    No,
}

impl<T> Default for Choice<T> {
    fn default() -> Self {
        Self::No
    }
}

#[derive(Default, Clone)]
pub struct TokenizerConfig {
    parse_char_as_string: bool,
    allow_digit_separator: Choice<char>,
}

#[derive(Clone)]
pub struct TokenizerBuilder {
    conf: TokenizerConfig,
}

impl TokenizerBuilder {
    pub fn new() -> TokenizerBuilder {
        TokenizerBuilder {
            conf: TokenizerConfig::default(),
        }
    }

    pub fn parse_char_as_string(self, set_to: bool) -> Self {
        let mut lb = TokenizerBuilder::new();
        lb.conf = self.conf;
        lb.conf.parse_char_as_string = set_to;
        lb
    }

    pub fn allow_digit_separator(self, choice: Choice<char>) -> Self {
        let mut lb = TokenizerBuilder::new();
        lb.conf = self.conf;
        lb.conf.allow_digit_separator = choice;
        lb
    }

    pub fn build<T>(self, with_input: T) -> Tokenizer
    where
        T: ToString,
    {
        Tokenizer::new(with_input, self.conf)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutOfBound {
    Empty,
    Out,
    Within,
}

#[derive(thiserror::Error, Debug)]
pub enum TokenizationError {
    #[error("No valid char at {0}")]
    NotAValidChar(Loc),
    #[error("Unexpected digit separator at {0}")]
    UnexpectedDigitSeparator(Loc),
}

impl Tokenizer {
    pub fn builder() -> TokenizerBuilder {
        TokenizerBuilder::new()
    }
    pub fn new<T>(input: T, config: TokenizerConfig) -> Self
    where
        T: ToString,
    {
        let input = input.to_string();
        Self {
            lines: input.lines().map(|line| line.chars().collect()).collect(),
            ln: 0,
            col: 0,
            config,
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
    }

    fn is_out_of_bound_for(&self, ln: usize, col: usize) -> OutOfBound {
        if ln >= self.lines.len() {
            return OutOfBound::Out;
        } else if !self.lines[ln].is_empty() {
            if col >= self.lines[self.ln].len() {
                return OutOfBound::Out;
            }
        } else {
            return OutOfBound::Empty;
        }

        OutOfBound::Within
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
                    break;
                }

                self.ln += 1;
            } else {
                let col_max = self.lines[self.ln].len();
                if self.col + 1 >= col_max {
                    self.next_line();
                } else {
                    self.col += 1;
                }
            }

            len -= 1;
        }
    }

    fn fake_consume(&self, len: usize) -> (OutOfBound, Loc) {
        let mut len = len;
        let ln_max = self.lines.len();

        let mut ln = self.ln;
        let mut col = self.col;

        while len > 0 {
            if ln + 1 > ln_max {
                ln += 1;
            } else {
                let col_max = self.lines[ln].len();
                if col + 1 >= col_max {
                    ln += 1;
                    col = 0;
                } else {
                    col += 1;
                }
            }

            len -= 1;
        }

        (self.is_out_of_bound_for(ln, col), Loc(ln, col))
    }

    #[inline]
    fn next_line(&mut self) {
        self.ln += 1;
        self.col = 0;
    }

    fn get_next_char(&self) -> Option<&char> {
        if self.is_out_of_bound() == OutOfBound::Within {
            Some(&self.lines[self.ln][self.col])
        } else {
            None
        }
    }

    fn peek_tok(&self) -> Option<&char> {
        let (bound, loc) = self.fake_consume(1);
        match bound {
            OutOfBound::Empty => Some(&'\n'),
            OutOfBound::Out => None,
            OutOfBound::Within => Some(&self.lines[loc.0][loc.1]),
        }
    }

    fn parse_word(&mut self) -> Result<Token, TokenizationError> {
        let mut word = String::new();
        let start_ln = self.ln;
        let start_col = self.col;
        while let Some(c) = self.get_next_char() {
            if *c != ' ' {
                word.push(*c);
            } else {
                break;
            }
            self.consume(1);
        }

        Ok(Token {
            r#type: TokenType::Word,
            value: Box::new(word),
            loc: Loc(start_ln, start_col),
        })
    }

    fn parse_float(&mut self) -> Result<Token, TokenizationError> {
        let mut float = String::new();
        let start_ln = self.ln;
        let start_col = self.col;
        let mut encountered_dot = false;

        if *self.get_next_char().unwrap() == '.' {
            float.push_str("0.");
            encountered_dot = true;
            self.consume(1);
        }

        while let Some(c) = self.get_next_char() {
            if c.is_ascii_digit() {
                float.push(*c);
            } else if *c == '.' {
                if encountered_dot {
                    break;
                } else {
                    float.push('.');
                    encountered_dot = true;
                }
            } else {
                break;
            }
            self.consume(1);
        }

        Ok(Token {
            r#type: TokenType::Number(NumberType::Float),
            value: Box::new(float),
            loc: Loc(start_ln, start_col),
        })
    }

    fn parse_number(&mut self) -> Result<Token, TokenizationError> {
        let mut num_type = NumberType::Seq;
        let mut parsing_float = false;
        let mut num = String::new();

        let start_ln = self.ln;
        let start_col = self.col;
        let mut inner_col = self.col;

        while let Some(c) = self.get_next_char() {
            if c.is_ascii_digit() {
                num.push(*c);
            } else if *c == '.' {
                if parsing_float {
                    break;
                } else {
                    parsing_float = true;
                    num.push('.');
                    num_type = NumberType::Float
                }
            } else if let Choice::Yes(with) = &self.config.allow_digit_separator {
                if *c == *with {
                    self.consume(1);
                    if let Some(next_char) = self.get_next_char() {
                        if !next_char.is_ascii_digit() {
                            return Err(TokenizationError::UnexpectedDigitSeparator(Loc(
                                start_ln, inner_col,
                            )));
                        } else {
                            num.push(*next_char);
                        }
                    } else {
                        return Err(TokenizationError::UnexpectedDigitSeparator(Loc(
                            start_ln, inner_col,
                        )));
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
            inner_col += 1;
            self.consume(1);
        }

        Ok(Token {
            r#type: TokenType::Number(num_type),
            value: Box::new(num),
            loc: Loc(start_ln, start_col),
        })
    }

    fn parse_binary(&mut self) -> Result<Token, TokenizationError> {
        let mut num = String::new();

        let start_ln = self.ln;
        let start_col = self.col;
        self.consume(2);

        while let Some(c) = self.get_next_char() {
            if matches!(*c, '1' | '0') {
                num.push(*c);
            } else {
                break;
            }
            self.consume(1);
        }

        Ok(Token {
            r#type: TokenType::Number(NumberType::Binary),
            value: Box::new(num),
            loc: Loc(start_ln, start_col),
        })
    }

    fn parse_hex(&mut self) -> Result<Token, TokenizationError> {
        let mut num = String::new();

        let start_ln = self.ln;
        let start_col = self.col;
        self.consume(2);

        while let Some(c) = self.get_next_char() {
            if c.is_ascii_hexdigit() {
                num.push(*c);
            } else {
                break;
            }
            self.consume(1);
        }

        Ok(Token {
            r#type: TokenType::Number(NumberType::Hex),
            value: Box::new(num),
            loc: Loc(start_ln, start_col),
        })
    }

    fn parse_octal(&mut self) -> Result<Token, TokenizationError> {
        let mut num = String::new();

        let start_ln = self.ln;
        let start_col = self.col;
        self.consume(2);

        while let Some(c) = self.get_next_char() {
            if matches!(*c, '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7') {
                num.push(*c);
            } else {
                break;
            }
            self.consume(1);
        }

        Ok(Token {
            r#type: TokenType::Number(NumberType::Octal),
            value: Box::new(num),
            loc: Loc(start_ln, start_col),
        })
    }

    fn parse_string(&mut self) -> Result<Token, TokenizationError> {
        let mut string = String::new();
        let mut is_escaped = false;
        let start_ln = self.ln;
        let start_col = self.col;

        self.consume(1);

        while let Some(c) = self.get_next_char() {
            if *c == '"' && !is_escaped {
                self.consume(1);
                break;
            }
            if *c == '\\' && !is_escaped {
                is_escaped = true;
                self.consume(1);
                continue;
            }
            string.push(*c);
            is_escaped = false;
            self.consume(1);
        }

        Ok(Token {
            r#type: TokenType::String,
            value: Box::new(string),
            loc: Loc(start_ln, start_col),
        })
    }

    fn parse_char(&mut self) -> Result<Token, TokenizationError> {
        let mut chr = String::new();
        let mut is_escaped = false;
        let start_ln = self.ln;
        let start_col = self.col;

        self.consume(1);

        while let Some(c) = self.get_next_char() {
            if *c == '\'' && !is_escaped {
                self.consume(1);
                break;
            }
            if *c == '\\' && !is_escaped {
                is_escaped = true;
                self.consume(1);
                continue;
            }
            chr.push(*c);
            is_escaped = false;
            self.consume(1);
        }

        let out_type = if !self.config.parse_char_as_string {
            TokenType::Char
        } else {
            TokenType::String
        };

        if out_type == TokenType::Char && chr.len() > 1 {
            Err(TokenizationError::NotAValidChar(Loc(start_ln, start_col)))
        } else {
            Ok(Token {
                r#type: out_type,
                value: Box::new(chr),
                loc: Loc(start_ln, start_col),
            })
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, TokenizationError> {
        let mut tokens = vec![];
        while self.is_out_of_bound() != OutOfBound::Out {
            if self.is_out_of_bound() == OutOfBound::Empty {
                self.next_line();
            } else {
                let next_char = self.get_next_char().unwrap();
                if matches!(next_char, ' ') {
                    self.consume(1);
                } else if matches!(next_char, '0'..='9') {
                    let first_digit = *self.get_next_char().unwrap();
                    if first_digit == '0' {
                        if let Some(c) = self.peek_tok() {
                            match *c {
                                'x' => {
                                    tokens.push(self.parse_hex()?);
                                }
                                'o' => {
                                    tokens.push(self.parse_octal()?);
                                }
                                'b' => {
                                    tokens.push(self.parse_binary()?);
                                }
                                '.' => {
                                    tokens.push(self.parse_float()?);
                                }
                                _ => {
                                    tokens.push(self.parse_number()?);
                                }
                            };
                        } else {
                            tokens.push(self.parse_number()?);
                        }
                    } else {
                        tokens.push(self.parse_number()?);
                    }
                } else if matches!(next_char, '.') {
                    if let Some(c) = self.peek_tok() {
                        if *c == '\n' {
                            self.consume(1);
                        } else if c.is_ascii_digit() {
                            tokens.push(self.parse_float()?);
                        }
                    } else {
                        tokens.push(Token {
                            r#type: TokenType::Symbol,
                            value: Box::new(".".into()),
                            loc: Loc(self.ln, self.col),
                        });
                        self.consume(1);
                    }
                } else if matches!(next_char, '"') {
                    tokens.push(self.parse_string()?);
                } else if matches!(next_char, '\'') {
                    tokens.push(self.parse_char()?);
                } else {
                    tokens.push(self.parse_word()?)
                }
            }
        }

        Ok(tokens)
    }
}
