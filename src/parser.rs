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
pub struct ParseError {
	text: &'static str,
}

// does stringify!() always return something that can be passed to concat!() ?
macro_rules! _try_parse_field {
	($iter:ident, $kind:ident) => {
		match $iter.next().ok_or(parse_eof_expected!($kind))?.value {
			TokenValue::$kind(field) => field,
			_ => throw_parse_expected!($kind),
		}
	};
}

macro_rules! _try_parse_empty {
	($iter:ident, $kind:ident) => {
		match $iter.next().ok_or(parse_eof_expected!($kind))?.value {
			TokenValue::$kind => (),
			_ => throw_parse_expected!($kind),
		}
	};
}

macro_rules! try_parse {
	($iter:ident, LPAR) => (_try_parse_empty!($iter, LPAR));
	($iter:ident, RPAR) => (_try_parse_empty!($iter, RPAR));
	($iter:ident, LBR) => (_try_parse_empty!($iter, LBR));
	($iter:ident, RBR) => (_try_parse_empty!($iter, RBR));
	($iter:ident, COMMA) => (_try_parse_empty!($iter, COMMA));
	($iter:ident, AMP) => (_try_parse_empty!($iter, AMP));
	($iter:ident, PROC) => (_try_parse_empty!($iter, PROC));
	($iter:ident, FN) => (_try_parse_empty!($iter, FN));
	($iter:ident, LABEL) => (_try_parse_empty!($iter, LABEL));
	($iter:ident, OP) => (_try_parse_empty!($iter, OP));
	($iter:ident, VAR) => (_try_parse_empty!($iter, VAR));
	($iter:ident, IN) => (_try_parse_empty!($iter, IN));
	($iter:ident, OUT) => (_try_parse_empty!($iter, OUT));
	($iter:ident, NAME) => (_try_parse_field!($iter, NAME));
	($iter:ident, INT) => (_try_parse_field!($iter, INT));
	
}

macro_rules! comma_or_sep {
	($i:ident) => (stringify!($i));
	($i1:ident, $i2:ident) => (concat!(stringify!($i1), " or ", stringify!($i2)));
	($i1:ident, $i2:ident, $i3:ident) => (concat!(stringify!($i1), ", ", stringify!($i2), ", or ", stringify!($i3)));
	($first:ident, $($rest:ident),*) => (concat!(stringify!($first), ", ", comma_or_sep!($($rest),*)));
}

macro_rules! parse_expected {
	($($kinds:ident),*) => (ParseError {text: concat!("parse error: expected ", comma_or_sep!($($kinds),*))});
}

macro_rules! parse_eof_expected {
	($($kinds:ident),*) => (ParseError {text: concat!("parse error: at EOF, expected ", comma_or_sep!($($kinds),*))});
}

macro_rules! throw_parse_expected {
	($($kinds:ident),*) => (Err(parse_expected!($($kinds),*))?);
}

macro_rules! throw_parse_eof_expected {
	($($kinds:ident),*) => (Err(parse_eof_expected!($($kinds),*))?);
}

pub fn parse<'a, I: Iterator<Item = Token<'a>>>(iter: I) -> Result<CodeNode<'a>, ParseError> {
	parse_code(&mut iter.peekable())
}

fn parse_code<'a, I: Iterator<Item = Token<'a>>>(iter: &mut Peekable<I>) -> Result<CodeNode<'a>, ParseError> {
	// use parse_stmt to parse stmts into a Vec<StmtNode>
	// then pass it into Visitor::code()
	let mut items: Vec<ItemNode<'a>> = Vec::new();
	while let Some(item) = parse_item(iter)? {
		items.push(item);
	}
	Ok(CodeNode {code: items})
}

fn parse_item<'a, I: Iterator<Item = Token<'a>>>(iter: &mut Peekable<I>) -> Result<Option<ItemNode<'a>>, ParseError> {
	// check next token
	// must be proc (for now, will add fn, possibly others)
	match match iter.next() {None => return Ok(None), Some(x) => x}.value {
		TokenValue::PROC => {
			// parse name '(' args ')' '{' code '}'
			// jumptargets will come later
			let name = try_parse!(iter, NAME);
			try_parse!(iter, LPAR);
			let mut args: Vec<ProcArgNode> = Vec::new();
			while let Some(arg) = parse_procarg(iter)? {
				args.push(arg);
				match iter.next().ok_or(parse_eof_expected!(COMMA, RPAR))?.value {
					TokenValue::COMMA => (),
					TokenValue::RPAR => break,
					_ => throw_parse_expected!(COMMA, RPAR),
				}
			}
			try_parse!(iter, LBR);
			let mut stmts: Vec<StmtNode<'a>> = Vec::new();
			while let Some(stmt) = parse_stmt(iter)? {
				stmts.push(stmt);
			}
			try_parse!(iter, RBR);
			Ok(Some(ItemNode::Proc(ProcNode {name: name, args: args, code: stmts})))
		},
		_ => throw_parse_expected!(PROC, EOF),
	}
}

fn parse_procarg<'a, I: Iterator<Item = Token<'a>>>(iter: &mut Peekable<I>) -> Result<Option<ProcArgNode<'a>>, ParseError> {
	// uhh can have in out
	// then name type
	// ok so pop tokens until a name comes up
	let mut in_ = false;
	let mut out = false;
	let name = loop {
		match iter.next().ok_or(parse_eof_expected!(IN, OUT, NAME))?.value {
			TokenValue::IN => {in_ = true;},
			TokenValue::OUT => {out = true;},
			TokenValue::NAME(name) => {break name;},
			_ => throw_parse_expected!(IN, OUT, NAME),
		}
	};
	let argtype = parse_argtype(iter)?;
	Ok(Some(ProcArgNode {in_: in_, out: out, name: name, argtype: argtype}))
}

fn parse_argtype<'a, I: Iterator<Item = Token<'a>>>(iter: &mut Peekable<I>) -> Result<ArgTypeNode, ParseError> {
	let size = try_parse!(iter, INT);
	Ok(ArgTypeNode {size: size})
}

fn parse_stmt<'a, I: Iterator<Item = Token<'a>>>(iter: &mut Peekable<I>) -> Result<Option<StmtNode<'a>>, ParseError> {
	match iter.peek().ok_or(parse_eof_expected!(LABEL, OP, VAR, RBR))?.value {
		TokenValue::LABEL => {
			iter.next();
			let name = try_parse!(iter, NAME);
			Ok(Some(StmtNode::Label(LabelNode {name: name})))
		},
		TokenValue::OP => {
			iter.next();
			let name = try_parse!(iter, NAME);
			let mut args: Vec<ValueNode<'a>> = Vec::new();
			while let Some(arg) = parse_value(iter)? {
				args.push(arg);
			}
			Ok(Some(StmtNode::Op(OpNode {name: name, args: args})))
		},
		TokenValue::VAR => {
			iter.next();
			let name = try_parse!(iter, NAME);
			let vartype = parse_vartype(iter)?;
			Ok(Some(StmtNode::Var(VarNode {name: name, vartype})))
		},
		TokenValue::RBR => Ok(None),
		_ => throw_parse_eof_expected!(LABEL, OP, VAR, RBR),
	}
}

fn parse_vartype<'a, I: Iterator<Item = Token<'a>>>(iter: &mut Peekable<I>) -> Result<VarTypeNode, ParseError> {
	let size = try_parse!(iter, INT);
	Ok(VarTypeNode {size: size})
}

fn parse_value<'a, I: Iterator<Item = Token<'a>>>(iter: &mut Peekable<I>) -> Result<Option<ValueNode<'a>>, ParseError> {
	match iter.peek().ok_or(parse_eof_expected!(NAME, INT))?.value {
		TokenValue::NAME(name) => {
			iter.next();
			Ok(Some(ValueNode::Name(name)))
		},
		TokenValue::INT(n) => {
			iter.next();
			Ok(Some(ValueNode::Int(n)))
		},
		_ => Ok(None),
	}
}

#[derive(Debug)]
pub struct CodeNode<'a> {
	pub code: Vec<ItemNode<'a>>,
}

#[derive(Debug)]
pub enum ItemNode<'a> {
	Proc(ProcNode<'a>),
}

#[derive(Debug)]
pub struct ProcNode<'a> {
	pub name: &'a str,
	pub args: Vec<ProcArgNode<'a>>,
	pub code: Vec<StmtNode<'a>>,
}

impl<'a> ItemNode<'a> {
	pub fn get_name(&self) -> &'a str {
		match &self {
			ItemNode::Proc(proc) => proc.name,
		}
	}
}

#[derive(Debug)]
pub enum StmtNode<'a> {
	Label(LabelNode<'a>),
	Op(OpNode<'a>),
	Var(VarNode<'a>),
}

#[derive(Debug)]
pub struct LabelNode<'a> {
	pub name: &'a str,
}

#[derive(Debug)]
pub struct OpNode<'a> {
	pub name: &'a str,
	pub args: Vec<ValueNode<'a>>,
}

#[derive(Debug)]
pub enum ValueNode<'a> {
	Name(&'a str),
	Int(i32),
}

#[derive(Debug)]
pub struct VarNode<'a> {
	pub name: &'a str,
	pub vartype: VarTypeNode,
}

#[derive(Debug)]
pub struct ProcArgNode<'a> {
	pub in_: bool,
	pub out: bool,
	pub name: &'a str,
	pub argtype: ArgTypeNode,
}

#[derive(Debug)]
pub struct VarTypeNode { // will probably have more fields later
	pub size: i32,
}

#[derive(Debug)]
pub struct ArgTypeNode { // will probably have more fields later
	pub size: i32,
}