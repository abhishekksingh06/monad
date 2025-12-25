use syntax::lexer::lex;

fn main() {
    let tokens = lex(1, "@");
    println!("{:#?}", tokens)
}
