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
	pub targets: Vec<Target>, // indexes into the parent Proc's code // probably means this has to be a private type maybe // also has to match the number of targets given by the Proc or builtin it's referencing
}

#[derive(Debug)]
pub struct Target {
	pub target: usize,
	pub vars: Vec<usize>,
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
	IFZ,
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

#[derive(Debug)]
pub struct CompileError {
	pub text: &'static str,
}

pub fn compile<'a>(code: &CodeNode<'a>) -> Result<(Compiled, HashMap<&'a str, usize>), CompileError> { // returns an executable code object and a mapping of names to callables
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
			Some(_) => Err(CompileError {text: "duplicate proc name :((("})?,
			None => (),
		}
		callables.push(item);
	}
	Ok((Compiled {callables: {
		let mut compiled: Vec<Callable> = Vec::new();
		for callable in callables {
			compiled.push(compile_callable(callable, &callablemap)?)
		}
		compiled
	}}, callablemap))
}

pub fn compile_callable<'a>(item: &ItemNode<'a>, callablemap: &HashMap<&str, usize>) -> Result<Callable, CompileError> {
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
			let mut stmtcount = 0;
			for stmt in proc.code.iter() {
				match stmt {
					StmtNode::Label(label) => {
						match labelmap.insert(label.name, stmtcount) {
							Some(_) => Err(CompileError {text: "duplicate label name :((("})?,
							None => (),
						}
					},
					_ => {stmtcount += 1;},
				}
			}
			let mut ops: Vec<Op> = Vec::new();
			for stmt in &proc.code {
				match stmt {
					StmtNode::Label(_) => (),
					StmtNode::Op(op) => {
						ops.push(Op {
							name: get_op(op.name, callablemap),
							args: {
								let mut args: Vec<Arg> = Vec::new();
								for arg in &op.args {
									args.push(match arg {
										ValueNode::Name(name) => Arg::Var(*varmap.get(name).ok_or(CompileError {text: "reference to nonexistent variable"})?),
										ValueNode::Int(n) => Arg::Int(*n),
									});
								}
								args
							},
							targets: if op.targets.len() == 0 {
								vec!(Target {target: ops.len() + 1, vars: vec!()})
							} else {
								let mut targets: Vec<Target> = Vec::new();
								for target in &op.targets {
									targets.push(Target {
										target: *labelmap.get(target.name).ok_or(CompileError {text: "reference to nonexistent label"})?, vars: vec!()
									});
								}
								targets
							},
						});
					},
					StmtNode::Var(var) => {
						varmap.insert(var.name, vars.len());
						vars.push(Var {vartype: (&var.vartype).into()});
					},
				}
			}
			ops.push(Op {name: OpName::Builtin(Builtin::RET), args: vec!(), targets: vec!()});
			Ok(Callable::Proc(Proc {
				argcount: proc.args.len(),
				vars: vars,
				code: ops,
			}))
		},
	}
}

fn get_op(name: &str, callablemap: &HashMap<&str, usize>) -> OpName {
	if let Some(i) = callablemap.get(name) {
		OpName::UserDef(*i)
	} else if name == "set" {
		OpName::Builtin(Builtin::SET)
	} else if name == "ifz" {
		OpName::Builtin(Builtin::IFZ)
	} else {
		panic!("what callable is {:?} ???", name);
	}
}