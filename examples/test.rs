use std::time::SystemTime;

use tinytoken::Tokenizer;

const TO_PARSE: &str = include_str!("./test.rs");

fn main() {
    let tokenizer = Tokenizer::builder()
        .parse_char_as_string(true)
        .allow_digit_separator(tinytoken::Choice::Yes('_'))
        .add_symbols(&['{', '}', '(', ')', ';', '#', ',', '[', ']'])
        .add_operators(&['+', '-', '*', '%', '/', '&'])
        .ignore_numbers(true)
        .build(TO_PARSE);
    // A little comment 77777.
    let start_time = SystemTime::now();
    match tokenizer.tokenize() {
        Ok(tokens) => {
            eprintln!(
                "-> elapsed: {}µs",
                start_time.elapsed().unwrap().as_micros()
            );
            println!("---------\nparsed {} token(s)\n---------", tokens.len());
            println!("{tokens:?}");
        }
        Err(e) => {
            println!("{e}")
        }
    }
}
