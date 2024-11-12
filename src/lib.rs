#![doc = include_str!("../README.md")]

use error::TokenizationError;

/// Contains error definitions specific to tokenization
pub mod error;

/// Represents the types of numeric tokens recognized by the tokenizer
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumberType {
    /// Floating-point numbers (e.g., `3.14`, `.25`)
    Float,
    /// Hexadecimal numbers (e.g., `0x1A3F`)
    Hex,
    /// Binary numbers (e.g., `0b1010`)
    Binary,
    /// Octal numbers (e.g., `0o755`)
    Octal,
    /// Sequential integers (e.g., `12345`)
    Seq,
}

/// Represents all possible token types that can be parsed by the tokenizer
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenType {
    /// Any alphanumeric string
    Word,
    /// A numeric token, where [NumberType] specifies the format
    Number(NumberType),
    /// A sequence of characters surrounded by double quotes ("example")
    String,
    /// A single character surrounded by single quotes ('a')
    Char,
    /// A character recognized as a symbol
    Symbol,
    /// A character recognized as an operator
    Operator,
}

/// Represents the location of a token in the input text, with line and column values
///
/// Format: Formats Loc as `<line+1>`:`<column+1>`.
#[derive(Clone, Copy, Debug)]
pub struct Loc(
    /// Line number (0-based index)
    pub usize,
    /// Column number (0-based index)
    pub usize,
);

impl std::fmt::Display for Loc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.0 + 1, self.1 + 1)
    }
}

/// Represents an individual token with type, value, and location
#[derive(Debug, Clone)]
pub struct Token {
    /// The [TokenType] of the token
    pub r#type: TokenType,
    /// The text of the token
    pub value: Box<String>,
    /// The location of the token in the input
    pub loc: Loc,
}

/// Primary struct for tokenizing an input string, with methods for parsing and generating tokens
pub struct Tokenizer {
    pub lines: Vec<Vec<char>>,
    ln: usize,
    col: usize,
    config: TokenizerConfig,
}

/// Configurable option for specific settings in [TokenizerConfig]
#[derive(Debug, Clone, Copy)]
pub enum Choice<T>
where
    T: Copy + Clone,
{
    /// An active choice with a specified value of type T
    Yes(T),
    /// No active choice
    No,
}

impl<T> Default for Choice<T>
where
    T: Copy + Clone,
{
    fn default() -> Self {
        Self::No
    }
}

/// Configuration struct for the tokenizer, allowing customization of tokenization behavior
#[derive(Default, Clone, Debug)]
pub struct TokenizerConfig {
    /// Whether single characters should be treated as strings and therefore may contains more than
    /// one character
    pub parse_char_as_string: bool,
    /// Considers numbers as words
    pub ignore_numbers: bool,
    /// Allows a specific character as a digit separator (e.g., `_`)
    pub allow_digit_separator: Choice<char>,
    /// List of characters to be treated as symbols
    pub consider_as_symbols: Vec<char>,
    /// List of characters to be treated as operators
    pub consider_as_operators: Vec<char>,
}

/// A builder struct for creating a [TokenizerConfig] instance with customized options
#[derive(Clone, Debug)]
pub struct TokenizerBuilder {
    conf: TokenizerConfig,
}

impl TokenizerBuilder {
    /// Creates a default [TokenizerBuilder]
    pub fn new() -> TokenizerBuilder {
        TokenizerBuilder {
            conf: TokenizerConfig {
                consider_as_symbols: vec!['.'],
                ..Default::default()
            },
        }
    }

    /// Configures character parsing behavior
    pub fn parse_char_as_string(self, set_to: bool) -> Self {
        let mut lb = TokenizerBuilder::new();
        lb.conf = self.conf;
        lb.conf.parse_char_as_string = set_to;
        lb
    }

    /// Configures number parsing behaviour
    pub fn ignore_numbers(self, set_to: bool) -> Self {
        let mut lb = TokenizerBuilder::new();
        lb.conf = self.conf;
        lb.conf.ignore_numbers = set_to;
        lb
    }

    /// Sets the digit separator
    pub fn allow_digit_separator(self, choice: Choice<char>) -> Self {
        let mut lb = TokenizerBuilder::new();
        lb.conf = self.conf;
        lb.conf.allow_digit_separator = choice;
        lb
    }

    /// Adds a symbol character
    pub fn add_symbol(self, sym: char) -> Self {
        let mut lb = TokenizerBuilder::new();
        lb.conf = self.conf;
        lb.conf.consider_as_symbols.push(sym);
        lb
    }

    /// Adds multiple symbol characters
    pub fn add_symbols(self, syms: &[char]) -> Self {
        let mut lb = TokenizerBuilder::new();
        lb.conf = self.conf;
        lb.conf.consider_as_symbols.extend(syms);
        lb
    }

    /// Adds an operator character
    pub fn add_operator(self, op: char) -> Self {
        let mut lb = TokenizerBuilder::new();
        lb.conf = self.conf;
        lb.conf.consider_as_operators.push(op);
        lb
    }

    /// Adds multiple operator characters
    pub fn add_operators(self, ops: &[char]) -> Self {
        let mut lb = TokenizerBuilder::new();
        lb.conf = self.conf;
        lb.conf.consider_as_operators.extend(ops);
        lb
    }

    /// Constructs a [Tokenizer] with the specified input and configuration.
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

impl Tokenizer {
    /// Creates a TokenizerBuilder instance for configuring and initializing the tokenizer
    pub fn builder() -> TokenizerBuilder {
        TokenizerBuilder::new()
    }
    /// Initializes the tokenizer with input text and a configuration
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
                if !self.config.consider_as_symbols.contains(c)
                    && !self.config.consider_as_operators.contains(c)
                {
                    word.push(*c);
                } else {
                    break;
                }
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

    fn parse_string(&mut self, delim: Option<char>) -> Result<Token, TokenizationError> {
        let mut string = String::new();
        let mut is_escaped = false;
        let start_ln = self.ln;
        let start_col = self.col;
        let delim = delim.unwrap_or('"');

        self.consume(1);

        while let Some(c) = self.get_next_char() {
            if *c == delim && !is_escaped {
                self.consume(1);
                break;
            }
            if *c == '\\' && !is_escaped {
                is_escaped = true;
                self.consume(1);
                continue;
            }
            if is_escaped {
                match *c {
                    'n' => {
                        string.push('\n');
                    }
                    '0' => {
                        string.push('\0');
                    }
                    't' => {
                        string.push('\t');
                    }
                    'r' => {
                        string.push('\r');
                    }
                    '\\' => {
                        string.push('\\');
                    }
                    _ => {
                        string.push_str(&format!("\\{c}"));
                    }
                }

                is_escaped = false;
            } else {
                string.push(*c);
            }
            self.consume(1);
        }

        Ok(Token {
            r#type: TokenType::String,
            value: Box::new(string),
            loc: Loc(start_ln, start_col),
        })
    }

    fn parse_char(&mut self) -> Result<Token, TokenizationError> {
        if self.config.parse_char_as_string {
            self.parse_string(Some('\''))
        } else {
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
    }

    /// Tokenizes the input and returns a list of Tokens or a [TokenizationError] if parsing fails
    pub fn tokenize(mut self) -> Result<Vec<Token>, TokenizationError> {
        let mut tokens = vec![];
        while self.is_out_of_bound() != OutOfBound::Out {
            if self.is_out_of_bound() == OutOfBound::Empty {
                self.next_line();
            } else {
                let next_char = self.get_next_char().unwrap();
                if matches!(next_char, ' ') {
                    self.consume(1);
                } else if matches!(next_char, '0'..='9') && !self.config.ignore_numbers {
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
                            tokens.push(Token {
                                r#type: TokenType::Symbol,
                                value: Box::new(".".into()),
                                loc: Loc(self.ln, self.col),
                            });
                            self.consume(1);
                        } else if c.is_ascii_digit() && !self.config.ignore_numbers {
                            tokens.push(self.parse_float()?);
                        } else {
                            tokens.push(Token {
                                r#type: TokenType::Symbol,
                                value: Box::new(".".into()),
                                loc: Loc(self.ln, self.col),
                            });
                            self.consume(1);
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
                    tokens.push(self.parse_string(None)?);
                } else if matches!(next_char, '\'') {
                    tokens.push(self.parse_char()?);
                } else if self.config.consider_as_symbols.contains(&next_char) {
                    tokens.push(Token {
                        r#type: TokenType::Symbol,
                        value: Box::new(next_char.to_string()),
                        loc: Loc(self.ln, self.col),
                    });
                    self.consume(1);
                } else if self.config.consider_as_operators.contains(&next_char) {
                    tokens.push(Token {
                        r#type: TokenType::Operator,
                        value: Box::new(next_char.to_string()),
                        loc: Loc(self.ln, self.col),
                    });
                    self.consume(1);
                } else {
                    tokens.push(self.parse_word()?)
                }
            }
        }

        Ok(tokens)
    }
}
