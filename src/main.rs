mod lexer;
mod parser;
mod compile;
mod run;

use lexer::Lexer;
use parser::parse;
use compile::compile;
use run::{execute, Value};

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
    let (compiled, symbols) = compile(&tree);
    println!("{:?} {:?}", compiled, symbols);
    let args: Vec<Value> = vec!(Value::new_i32(15), Value::new_i32(1));
    println!("{:?}", args);
    execute(&compiled, 0, &args);
    println!("{:?}", args);
}
