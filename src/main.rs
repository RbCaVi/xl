mod lexer;
mod parser;
mod compile;

use lexer::{Lexer, TokenValue};
use parser::parse;
use compile::compile;

fn main() {
    let content = r#"
proc swap (in out a 4, in out b 4) {
    var temp 4
    op set temp a
    op set a b
    op set b temp
}"#;
    println!("{}", content);
    let tree = match parse(Lexer::new(content).map(|x| {println!("{:?}", x.value); x})) {
        Err(_) => return,
        Ok(tree) => tree,
    };
    //println!("{:#?}", tree);
    let compiled = compile(&tree);
    println!("{:#?}", compiled);
}
