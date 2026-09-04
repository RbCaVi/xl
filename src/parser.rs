// ok the parser

// takes an iterator of tokens (usually the lexer but i Don't care)
// returns a parse tree
// or takes a lexer and visitor and applies the visitor to the parse tree
// anyway the grammar

// program = item*
// item = proc | fn
// proc = 'proc' name '(' procarg[','] ')' ['->' jumptarget[',']] '{' code '}'
// fn = 'fn' name '(' fnargs[','] ')' '{' code '}'
// code = stmt*
// stmt = label | op | var
// label = 'label' name
// op = 'op' name value*
// var = 'var' name type

// procarg = 'in'? 'out'? name type
// value = name | int

// type = int // for now

// let's start with just proc, op, var

// do a visitor

// i should also use Result s to avoid panic!() ing on parse errors
// later (tm)

use std::iter::Peekable;
use crate::lexer::{Token, TokenValue};

// does stringify!() always return something that can be passed to concat!() ?
macro_rules! try_parse {
	($iter:ident, $kind:ident, $field:ident) => {
		match $iter.next().expect(concat!("parse error: at EOF, expected ", stringify!($kind))).value {
			TokenValue::$kind($field) => $field,
			_ => {
				panic!(concat!("parse error: expected ", stringify!($kind)));
			}
		}
	};
	($iter:ident, $kind:ident) => {
		match $iter.next().expect(concat!("parse error: at EOF, expected ", stringify!($kind))).value {
			TokenValue::$kind => (),
			_ => {
				panic!(concat!("parse error: expected ", stringify!($kind)));
			}
		}
	};
}
pub fn parse<'a, I: Iterator<Item = Token<'a>>>(iter: I) -> CodeNode<'a> {
	parse_code(&mut iter.peekable())
}

fn parse_code<'a, I: Iterator<Item = Token<'a>>>(iter: &mut Peekable<I>) -> CodeNode<'a> {
	// use parse_stmt to parse stmts into a Vec<StmtNode>
	// then pass it into Visitor::code()
	let mut items: Vec<ItemNode<'a>> = Vec::new();
	while let Some(item) = parse_item(iter) {
		items.push(item);
	}
	CodeNode {code: items}
}

fn parse_item<'a, I: Iterator<Item = Token<'a>>>(iter: &mut Peekable<I>) -> Option<ItemNode<'a>> {
	// check next token
	// must be proc (for now, will add fn, possibly others)
	match iter.next()?.value {
		TokenValue::PROC => {
			// parse name '(' args ')' '{' code '}'
			// jumptargets will come later
			let name = try_parse!(iter, NAME, name);
			try_parse!(iter, LPAR);
			let mut args: Vec<ProcArgNode> = Vec::new();
			while let Some(arg) = parse_procarg(iter) {
				args.push(arg);
				match iter.next().expect("parse error: at EOF, expected COMMA or RPAR").value {
					TokenValue::COMMA => (),
					TokenValue::RPAR => break,
					_ => {
						panic!("parse error: expected COMMA or RPAR");
					}
				}
			}
			try_parse!(iter, LBR);
			let mut stmts: Vec<StmtNode<'a>> = Vec::new();
			while let Some(stmt) = parse_stmt(iter) {
				stmts.push(stmt);
			}
			try_parse!(iter, RBR);
			Some(ItemNode::Proc(ProcNode {name: name, args: args, code: stmts}))
		},
		_ => {
			panic!("parse error: expected PROC or EOF");
		},
	}
}

fn parse_procarg<'a, I: Iterator<Item = Token<'a>>>(iter: &mut Peekable<I>) -> Option<ProcArgNode<'a>> {
	// uhh can have in out
	// then name type
	// ok so pop tokens until a name comes up
	let mut in_ = false;
	let mut out = false;
	let name = loop {
		match iter.next().expect("parse error: at EOF, expected IN, OUT, or NAME").value {
			TokenValue::IN => {in_ = true;},
			TokenValue::OUT => {out = true;},
			TokenValue::NAME(name) => {break name;},
			_ => {
				panic!("parse error: expected IN, OUT, or NAME");
			}
		}
	};
	let argtype = parse_argtype(iter);
	Some(ProcArgNode {in_: in_, out: out, name: name, argtype: argtype})
}

fn parse_argtype<'a, I: Iterator<Item = Token<'a>>>(iter: &mut Peekable<I>) -> ArgTypeNode {
	let n = try_parse!(iter, INT, n);
	ArgTypeNode {n: n}
}

fn parse_stmt<'a, I: Iterator<Item = Token<'a>>>(iter: &mut Peekable<I>) -> Option<StmtNode<'a>> {
	match iter.peek().expect("parse error: at EOF, expected LABEL, OP, VAR, or RBR").value {
		TokenValue::LABEL => {
			iter.next();
			let name = try_parse!(iter, NAME, name);
			Some(StmtNode::Label(LabelNode {name: name}))
		},
		TokenValue::OP => {
			iter.next();
			let name = try_parse!(iter, NAME, name);
			let mut args: Vec<ValueNode<'a>> = Vec::new();
			while let Some(arg) = parse_value(iter) {
				args.push(arg);
			}
			Some(StmtNode::Op(OpNode {name: name, args: args}))
		},
		TokenValue::VAR => {
			iter.next();
			let name = try_parse!(iter, NAME, name);
			let vartype = parse_vartype(iter);
			Some(StmtNode::Var(VarNode {name: name, vartype}))
		},
		TokenValue::RBR => None,
		_ => {
			panic!("parse error: expected LABEL, OP, VAR, or RBR");
		},
	}
}

fn parse_vartype<'a, I: Iterator<Item = Token<'a>>>(iter: &mut Peekable<I>) -> VarTypeNode {
	let n = try_parse!(iter, INT, n);
	VarTypeNode {n: n}
}

fn parse_value<'a, I: Iterator<Item = Token<'a>>>(iter: &mut Peekable<I>) -> Option<ValueNode<'a>> {
	match iter.peek().expect("parse error: at EOF, expected NAME or INT").value {
		TokenValue::NAME(name) => {
			iter.next();
			Some(ValueNode::Name(name))
		},
		TokenValue::INT(n) => {
			iter.next();
			Some(ValueNode::Int(n))
		},
		_ => None,
	}
}

#[derive(Debug)]
pub struct CodeNode<'a> {
	code: Vec<ItemNode<'a>>,
}

#[derive(Debug)]
pub enum ItemNode<'a> {
	Proc(ProcNode<'a>),
}

#[derive(Debug)]
pub struct ProcNode<'a> {
	name: &'a str,
	args: Vec<ProcArgNode<'a>>,
	code: Vec<StmtNode<'a>>,
}

#[derive(Debug)]
pub enum StmtNode<'a> {
	Label(LabelNode<'a>),
	Op(OpNode<'a>),
	Var(VarNode<'a>),
}

#[derive(Debug)]
pub struct LabelNode<'a> {
	name: &'a str,
}

#[derive(Debug)]
pub struct OpNode<'a> {
	name: &'a str,
	args: Vec<ValueNode<'a>>,
}

#[derive(Debug)]
pub enum ValueNode<'a> {
	Name(&'a str),
	Int(i32),
}

#[derive(Debug)]
pub struct VarNode<'a> {
	name: &'a str,
	vartype: VarTypeNode,
}

#[derive(Debug)]
pub struct ProcArgNode<'a> {
	in_: bool,
	out: bool,
	name: &'a str,
	argtype: ArgTypeNode,
}

#[derive(Debug)]
pub struct VarTypeNode { // will probably have more fields later
	n: i32,
}

#[derive(Debug)]
pub struct ArgTypeNode { // will probably have more fields later
	n: i32,
}