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

#[derive(Debug)]
enum Item<P> {
	Proc(P),
}

#[derive(Debug)]
enum Stmt<L, O, V> {
	Label(L),
	Op(O),
	Var(V),
}

#[derive(Debug)]
enum Value<N, I> {
	Name(N),
	Int(I),
}

// i don't know if there is a better way to
// add forced associated types
// because putting type ## = ##; in either
// a trait definition or trait impl is
// apparently an unstable feature
trait VisitorTypes<'a> {
	type Item;
	type Stmt;
	type Value;
}

impl<'a, V: ?Sized + Visitor<'a>> VisitorTypes<'a> for V {
	type Item = Item<V::Proc>;
	type Stmt = Stmt<V::Label, V::Op, V::Var>;
	type Value = Value<V::ValueName, V::Int>;
}

pub trait Visitor<'a> {
	type Code;
	type Proc;
	type Label;
	type Op;
	type Var;
	type ProcArg;
	type ValueName;
	type Int;
	type VarType;
	type ArgType;

	// ugly ahh type
	fn code(&mut self, code: Vec<<Self as VisitorTypes<'a>>::Item>) -> Self::Code;

	fn proc(&mut self, name: &'a str, args: Vec<Self::ProcArg>, code: Vec<<Self as VisitorTypes<'a>>::Stmt>) -> Self::Proc;

	fn label(&mut self, name: &'a str) -> Self::Label;
	fn op(&mut self, name: &'a str, args: Vec<<Self as VisitorTypes<'a>>::Value>) -> Self::Op;
	fn var(&mut self, name: &'a str, vartype: Self::VarType) -> Self::Var;

	fn procarg(&mut self, in_: bool, out: bool, name: &'a str, argtype: Self::ArgType) -> Self::ProcArg;
	fn valuename(&mut self, name: &'a str) -> Self::ValueName;
	fn int(&mut self, n: i32) -> Self::Int;
	fn vartype(&mut self, n: i32) -> Self::VarType;
	fn argtype(&mut self, n: i32) -> Self::ArgType;
}

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

pub fn parse_default<'a, V: Visitor<'a> + Default, I: Iterator<Item = Token<'a>>>(iter: I) -> V::Code {
	parse_code(&mut <V as Default>::default(), &mut iter.peekable())
}

pub fn parse<'a, V: Visitor<'a>, I: Iterator<Item = Token<'a>>>(visitor: &mut V, iter: I) -> V::Code {
	parse_code(visitor, &mut iter.peekable())
}

pub fn parse_code<'a, V: Visitor<'a>, I: Iterator<Item = Token<'a>>>(visitor: &mut V, iter: &mut Peekable<I>) -> V::Code {
	// use parse_stmt to parse stmts into a Vec<Stmt>
	// then pass it into Visitor::code()
	let mut items: Vec<<V as VisitorTypes<'a>>::Item> = Vec::new();
	while let Some(item) = parse_item(visitor, iter) {
		items.push(item);
	}
	visitor.code(items)
}

fn parse_item<'a, V: Visitor<'a>, I: Iterator<Item = Token<'a>>>(visitor: &mut V, iter: &mut Peekable<I>) -> Option<<V as VisitorTypes<'a>>::Item> {
	// check next token
	// must be proc (for now, will add fn, possibly others)
	match iter.next()?.value {
		TokenValue::PROC => {
			// parse name '(' args ')' '{' code '}'
			// jumptargets will come later
			let name = try_parse!(iter, NAME, name);
			try_parse!(iter, LPAR);
			let mut args: Vec<V::ProcArg> = Vec::new();
			while let Some(arg) = parse_procarg(visitor, iter) {
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
			let mut stmts: Vec<<V as VisitorTypes<'a>>::Stmt> = Vec::new();
			while let Some(stmt) = parse_stmt(visitor, iter) {
				stmts.push(stmt);
			}
			try_parse!(iter, RBR);
			Some(Item::Proc(visitor.proc(name, args, stmts)))
		},
		_ => {
			panic!("parse error: expected PROC or EOF");
		},
	}
}

fn parse_procarg<'a, V: Visitor<'a>, I: Iterator<Item = Token<'a>>>(visitor: &mut V, iter: &mut Peekable<I>) -> Option<V::ProcArg> {
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
	let argtype = parse_argtype(visitor, iter);
	// should have eated the ',' or ')' after
	Some(visitor.procarg(in_, out, name, argtype))
}

fn parse_argtype<'a, V: Visitor<'a>, I: Iterator<Item = Token<'a>>>(visitor: &mut V, iter: &mut Peekable<I>) -> V::ArgType {
	let n = try_parse!(iter, INT, n);
	visitor.argtype(n)
}

fn parse_stmt<'a, V: Visitor<'a>, I: Iterator<Item = Token<'a>>>(visitor: &mut V, iter: &mut Peekable<I>) -> Option<<V as VisitorTypes<'a>>::Stmt> {
	match iter.peek().expect("parse error: at EOF, expected LABEL, OP, VAR, or RBR").value {
		TokenValue::LABEL => {
			iter.next();
			let name = try_parse!(iter, NAME, name);
			Some(Stmt::Label(visitor.label(name)))
		},
		TokenValue::OP => {
			iter.next();
			let name = try_parse!(iter, NAME, name);
			let mut values: Vec<<V as VisitorTypes<'a>>::Value> = Vec::new();
			while let Some(value) = parse_value(visitor, iter) {
				values.push(value);
			}
			Some(Stmt::Op(visitor.op(name, values)))
		},
		TokenValue::VAR => {
			iter.next();
			let name = try_parse!(iter, NAME, name);
			let vartype = parse_vartype(visitor, iter);
			Some(Stmt::Var(visitor.var(name, vartype)))
		},
		TokenValue::RBR => None,
		_ => {
			panic!("parse error: expected LABEL, OP, VAR, or RBR");
		},
	}
}

fn parse_vartype<'a, V: Visitor<'a>, I: Iterator<Item = Token<'a>>>(visitor: &mut V, iter: &mut Peekable<I>) -> V::VarType {
	let n = try_parse!(iter, INT, n);
	visitor.vartype(n)
}

fn parse_value<'a, V: Visitor<'a>, I: Iterator<Item = Token<'a>>>(visitor: &mut V, iter: &mut Peekable<I>) -> Option<<V as VisitorTypes<'a>>::Value> {
	match iter.peek().expect("parse error: at EOF, expected NAME or INT").value {
		TokenValue::NAME(name) => {
			iter.next();
			Some(Value::Name(visitor.valuename(name)))
		},
		TokenValue::INT(n) => {
			iter.next();
			Some(Value::Int(visitor.int(n)))
		},
		_ => None,
	}
}

pub struct BuildTree<'a> {
	_man: &'a (),
}

#[derive(Debug)]
pub struct CodeNode<'a> {
	code: Vec<<BuildTree<'a> as VisitorTypes<'a>>::Item>,
}

#[derive(Debug)]
pub struct ProcNode<'a> {
	name: &'a str,
	args: Vec<<BuildTree<'a> as Visitor<'a>>::ProcArg>,
	code: Vec<<BuildTree<'a> as VisitorTypes<'a>>::Stmt>,
}

#[derive(Debug)]
pub struct LabelNode<'a> {
	name: &'a str,
}

#[derive(Debug)]
pub struct OpNode<'a> {
	name: &'a str,
	args: Vec<<BuildTree<'a> as VisitorTypes<'a>>::Value>,
}

#[derive(Debug)]
pub struct VarNode<'a> {
	name: &'a str,
	vartype: <BuildTree<'a> as Visitor<'a>>::VarType,
}

#[derive(Debug)]
pub struct ProcArgNode<'a> {
	in_: bool,
	out: bool,
	name: &'a str,
	argtype: <BuildTree<'a> as Visitor<'a>>::ArgType,
}

#[derive(Debug)]
pub struct ValueNameNode<'a> {
	name: &'a str,
}

#[derive(Debug)]
pub struct IntNode {
	n: i32,
}

#[derive(Debug)]
pub struct VarTypeNode {
	n: i32,
}

#[derive(Debug)]
pub struct ArgTypeNode {
	n: i32,
}

static _man: () = ();

impl<'a> BuildTree<'a> {
	pub fn new() -> BuildTree<'a> {
		BuildTree {_man: &_man}
	}
}

impl<'a> Default for BuildTree<'a> {
	fn default() -> BuildTree<'a> {
		BuildTree::new()
	}
}

impl<'a> Visitor<'a> for BuildTree<'a> {
	type Code = CodeNode<'a>;
	type Proc = ProcNode<'a>;
	type Label = LabelNode<'a>;
	type Op = OpNode<'a>;
	type Var = VarNode<'a>;
	type ProcArg = ProcArgNode<'a>;
	type ValueName = ValueNameNode<'a>;
	type Int = IntNode;
	type VarType = VarTypeNode;
	type ArgType = ArgTypeNode;

	// ugly ahh type
	fn code(&mut self, code: Vec<<Self as VisitorTypes<'a>>::Item>) -> Self::Code {
		CodeNode {code: code}
	}

	fn proc(&mut self, name: &'a str, args: Vec<Self::ProcArg>, code: Vec<<Self as VisitorTypes<'a>>::Stmt>) -> Self::Proc {
		ProcNode {name: name, args: args, code: code}
	}

	fn label(&mut self, name: &'a str) -> Self::Label {
		LabelNode {name: name}
	}

	fn op(&mut self, name: &'a str, args: Vec<<Self as VisitorTypes<'a>>::Value>) -> Self::Op {
		OpNode {name: name, args: args}
	}

	fn var(&mut self, name: &'a str, vartype: Self::VarType) -> Self::Var {
		VarNode {name: name, vartype: vartype}
	}

	fn procarg(&mut self, in_: bool, out: bool, name: &'a str, argtype: Self::ArgType) -> Self::ProcArg {
		ProcArgNode {in_: in_, out: out, name: name, argtype: argtype}
	}

	fn valuename(&mut self, name: &'a str) -> Self::ValueName {
		ValueNameNode {name: name}
	}

	fn int(&mut self, n: i32) -> Self::Int {
		IntNode {n: n}
	}

	fn vartype(&mut self, n: i32) -> Self::VarType {
		VarTypeNode {n: n}
	}

	fn argtype(&mut self, n: i32) -> Self::ArgType {
		ArgTypeNode {n: n}
	}
}