// compile the syntax tree to... something

use std::collections::HashMap;
use crate::parser::{CodeNode, ItemNode, StmtNode, ArgTypeNode, VarTypeNode, ValueNode};

#[derive(Debug)]
pub struct Compiled {
	// what do i need here
	// procs
	// funcs when i add them
	// probably in a hashmap tbh
	// nah vec is faster frfr
	// idk bru h
	pub callables: Vec<Callable>,
}

#[derive(Debug)]
pub enum Callable {
	Proc(Proc),
}

#[derive(Debug)]
pub struct Proc {
	// uhhh code and vars?
	// how do the jumps work?
	// idk just index into the instructions array
	// who cares
	pub argcount: usize,
	pub vars: Vec<Var>,
	pub code: Vec<Op>,
}

#[derive(Debug)]
pub struct Op {
	pub name: OpName, // references a proc from the grandparent Compiled // so maybe Proc has to be private too? idk whatever // private constructors and mutability
	pub args: Vec<Arg>,
	pub targets: Vec<usize>, // indexes into the parent Proc's code // probably means this has to be a private type maybe // also has to match the number of targets given by the Proc or builtin it's referencing
}

#[derive(Debug)]
pub enum OpName {
	UserDef(usize),
	Builtin(Builtin),
}

#[derive(Debug)]
pub enum Builtin {
	RET,
	SET,
}

#[derive(Debug)]
pub enum Arg {
	Var(usize),
	Int(i32),
}

#[derive(Debug)]
pub struct Var {
	pub vartype: Type,
}

#[derive(Debug)]
pub struct Type {
	pub size: i32,
}

impl From<&ArgTypeNode> for Type {
	fn from(t: &ArgTypeNode) -> Type {
		Type {size: t.size}
	}
}

impl From<&VarTypeNode> for Type {
	fn from(t: &VarTypeNode) -> Type {
		Type {size: t.size}
	}
}

pub fn compile<'a>(code: &CodeNode<'a>) -> (Compiled, HashMap<&'a str, usize>) { // returns an executable code object and a mapping of names to callables
	// first find names
	// error on redefining a proc
	// loop through statements
	// match on type
	// hashset
	// or hashmap of name to &ItemNode
	// yeah
	let mut callablemap: HashMap<&str, usize> = HashMap::new();
	let mut callables: Vec<&ItemNode> = Vec::new();
	for item in &code.code {
		match callablemap.insert(item.get_name(), callables.len()) {
			Some(_) => panic!("duplicate proc name :((("),
			None => (),
		}
		callables.push(item);
	}
	(Compiled {callables: callables.into_iter().map(|c| compile_callable(c, &callablemap)).collect()}, callablemap)
}

pub fn compile_callable<'a>(item: &ItemNode<'a>, callablemap: &HashMap<&str, usize>) -> Callable {
	match item {
		ItemNode::Proc(proc) => {
			// it's possible to make a single pass variable resolver
			// probably
			// some deferred stuff or something
			// nah though
			// collect vars then process ops
			// actually do something like rust?
			// rebinding
			let mut varmap: HashMap<&str, usize> = HashMap::new();
			let mut vars: Vec<Var> = Vec::new();
			for arg in &proc.args {
				varmap.insert(arg.name, vars.len());
				vars.push(Var {vartype: (&arg.argtype).into()});
			}
			// oh yeah i need to grab labels
			// and i don't want to do a single pass method
			let mut labelmap: HashMap<&str, usize> = HashMap::new();
			for (i, stmt) in proc.code.iter().enumerate() {
				match stmt {
					StmtNode::Label(label) => {
						match labelmap.insert(label.name, i) {
							Some(_) => panic!("duplicate label name :(((((("),
							None => (),
						}
					},
					_ => (),
				}
			}
			let mut ops: Vec<Op> = Vec::new();
			for stmt in &proc.code {
				match stmt {
					StmtNode::Label(_) => (),
					StmtNode::Op(op) => {
						ops.push(Op {
							name: get_op(op.name, callablemap),
							args: op.args.iter().map(|arg| {
								match arg {
									ValueNode::Name(name) => Arg::Var(*varmap.get(name).unwrap()),
									ValueNode::Int(n) => Arg::Int(*n),
								}
							}).collect(),
							targets: vec!(ops.len() + 1),
						});
					},
					StmtNode::Var(var) => {
						varmap.insert(var.name, vars.len());
						vars.push(Var {vartype: (&var.vartype).into()});
					},
				}
			}
			ops.push(Op {name: OpName::Builtin(Builtin::RET), args: vec!(), targets: vec!()});
			Callable::Proc(Proc {
				argcount: proc.args.len(),
				vars: vars,
				code: ops,
			})
		},
	}
}

fn get_op(name: &str, callablemap: &HashMap<&str, usize>) -> OpName {
	if let Some(i) = callablemap.get(name) {
		OpName::UserDef(*i)
	} else if name == "set" {
		OpName::Builtin(Builtin::SET)
	} else {
		panic!("what callable is {:?} ???", name);
	}
}