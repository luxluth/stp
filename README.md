# Simple Tokenizer Preset (STP)

This library provides a tokenizer for parsing and categorizing different types
of tokens, such as words, numbers, strings, characters, symbols, and operators.
It includes configurable options to handle various tokenization rules and
formats, enabling fine-grained control over how text input is parsed.

## Example

```rust
use stp::{Tokenizer, TokenizerBuilder, Choice};

fn main() {
    let tokenizer = TokenizerBuilder::new()
        .parse_char_as_string(true)
        .allow_digit_separator(Choice::Yes('_'))
        .add_symbol('$')
        .add_operators(&['+', '-'])
        .build("let x = 123_456 + 0xFF");

    match tokenizer.tokenize() {
        Ok(tokens) => {
            for token in tokens {
                println!("{:?}", token);
            }
        }
        Err(err) => {
            eprintln!("Tokenization error: {err}");
        }
    }
}
```

## Contributions

Feel free to send a PR to improve and/or extend the tool capabilities
