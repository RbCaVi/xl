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

use crate::lexer::{Token};

enum Item<P> {
	Proc(P),
}

enum Stmt<L, O, V> {
	Label(L),
	Op(O),
	Var(V),
}

enum Value<N, I> {
	Name(N),
	Int(I),
}

// i don't know if there is a better way to
// add forced associated types
// because putting type ## = ##; in either
// a trait definition or trait impl is
// apparently an unstable feature
trait VisitorTypes {
	type Item;
	type Stmt;
	type Value;
}

impl<V: ?Sized + Visitor> VisitorTypes for V {
	type Item = Item<V::Proc>;
	type Stmt = Stmt<V::Label, V::Op, V::Var>;
	type Value = Value<V::ValueName, V::Int>;
}

trait Visitor {
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
	fn code(code: Vec<<Self as VisitorTypes>::Item>) -> Self::Code;

	fn proc(name: &str, args: Vec<Self::ProcArg>, code: Vec<<Self as VisitorTypes>::Stmt>) -> Self::Proc;

	fn label(name: &str) -> Self::Label;
	fn op(name: &str, args: Vec<<Self as VisitorTypes>::Value>) -> Self::Op;
	fn var(name: &str, vartype: Self::VarType) -> Self::Var;

	fn procarg(in_: bool, out: bool, name: &str, argtype: Self::ArgType) -> Self::ProcArg;
	fn valuename(name: &str) -> Self::ValueName;
	fn int(n: i32) -> Self::Int;
	fn vartype(n: i32) -> Self::VarType;
	fn argtype(n: i32) -> Self::ArgType;
}

// consumes the iterator it is provided
impl<'a, I: Iterator<Item = Token<'a>>> Parser<'a, I> {
	pub fn parse<V: Visitor>(&mut self) -> V::Code {
		// use parse_stmt to parse stmts into a Vec<Stmt>
		// then pass it into Visitor::code()
		let mut items: Vec<<V as VisitorTypes>::Item> = Vec::new();
		while let Some(item) = self.parse_item::<V>() {
			items.push(item);
		}
		V::code(items)
	}

	pub fn parse_item<V: Visitor>(&mut self) -> Option<<V as VisitorTypes>::Item> {
		todo!();
	}
}

struct Parser<'a, I: Iterator<Item = Token<'a>>> {
	iter: I,
}