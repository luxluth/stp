use stp::Tokenizer;

fn main() {
    let mut tokenizer = Tokenizer::builder().parse_char_as_string(true).build(
        "0b01010101000 0xFFFffFFF 0o4543431234 1324.4534543 3453987 .924894 ; 3 'あいしている'",
    );

    match tokenizer.tokenize() {
        Ok(tokens) => {
            eprintln!("---------\nparsed {} token(s)\n---------", tokens.len());
            eprintln!("{tokens:?}");
        }
        Err(e) => {
            eprintln!("{e}")
        }
    }
}
