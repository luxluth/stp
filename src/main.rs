use stp::Tokenizer;

fn main() {
    let mut tokenizer =
        Tokenizer::new("0b01010101000 0xFFFffFFF 0o4543431234 1324.4534543 3453987 .924894");
    let tokens = tokenizer.tokenize();
    eprintln!("---------\nparsed {} token(s)\n---------", tokens.len());
    eprintln!("{tokens:?}");
}
