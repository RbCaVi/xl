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
    for token in Lexer::new(content) {
        match token.value {
            TokenValue::LPAR => {println!("LPAR");}
            TokenValue::RPAR => {println!("RPAR");}
            TokenValue::LBR => {println!("LBR");}
            TokenValue::RBR => {println!("RBR");}
            TokenValue::COMMA => {println!("COMMA");}
            TokenValue::AMP => {println!("AMP");}
            TokenValue::PROC => {println!("PROC");}
            TokenValue::FN => {println!("FN");}
            TokenValue::LABEL => {println!("LABEL");}
            TokenValue::OP => {println!("OP");}
            TokenValue::VAR => {println!("VAR");}
            TokenValue::IN => {println!("IN");}
            TokenValue::OUT => {println!("OUT");}
            TokenValue::NAME(name) => {println!("NAME {}", name);}
            TokenValue::INT(i) => {println!("INT {}", i);}
        }
    }
    let tree = match parse(Lexer::new(content).map(|x| {println!("{:?}", x); x})) {
        Err(_) => return,
        Ok(tree) => tree,
    };
    println!("{:#?}", tree);
    let compiled = compile(&tree);
    println!("{:#?}", compiled);
}
