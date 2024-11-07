use std::time::SystemTime;

use stp::Tokenizer;

const TO_PARSE: &str = include_str!("./test.rs");

fn main() {
    let mut tokenizer = Tokenizer::builder()
        .parse_char_as_string(true)
        .allow_digit_separator(stp::Choice::Yes('_'))
        .add_symbols(&['{', '}', '(', ')', ';', '#', ',', '[', ']'])
        .add_operators(&['+', '-', '*', '%', '/', '&'])
        .build(TO_PARSE);
    let start_time = SystemTime::now();
    match tokenizer.tokenize() {
        Ok(tokens) => {
            eprintln!(
                "-> elapsed: {}µs",
                start_time.elapsed().unwrap().as_micros()
            );
            eprintln!("---------\nparsed {} token(s)\n---------", tokens.len());
            eprintln!("{tokens:?}");
        }
        Err(e) => {
            eprintln!("{e}")
        }
    }
}
