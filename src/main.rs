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
}
proc cswap (in out a 4, in out b 4, in c 4) {
    op ifz c -> l1, l2
    label l1
    op swap a b
    label l2
}
"#;
    println!("{}", content);
    let tree = match parse(Lexer::new(content).map(|x| {println!("{:?}", x.value); x})) {
        Err(err) => {println!("no tree {:?}", err); return;},
        Ok(tree) => tree,
    };
    //println!("{:#?}", tree);
    let (compiled, symbols) = compile(&tree);
    println!("{:?} {:?}", compiled, symbols);
    let args: Vec<Value> = vec!(Value::new_i32(15), Value::new_i32(1));
    println!("{:?}", args);
    execute(&compiled, *symbols.get("swap").unwrap(), &args);
    println!("{:?}", args);
    println!("");
    
    let args: Vec<Value> = vec!(Value::new_i32(15), Value::new_i32(1), Value::new_i32(1));
    println!("{:?}", args);
    execute(&compiled, *symbols.get("cswap").unwrap(), &args);
    println!("{:?}", args);
    println!("");
    
    let args: Vec<Value> = vec!(Value::new_i32(15), Value::new_i32(1), Value::new_i32(0));
    println!("{:?}", args);
    execute(&compiled, *symbols.get("cswap").unwrap(), &args);
    println!("{:?}", args);
    println!("");
    
}
