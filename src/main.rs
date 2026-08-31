mod lexer;
mod parser;

use lexer::{Lexer, TokenValue};
use crate::parser::{parse_default, BuildTree};

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
    println!("{:#?}", parse_default::<BuildTree, _>(Lexer::new(content).map(|x| {println!("{:?}", x); x})));
}
