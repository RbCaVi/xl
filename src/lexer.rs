// lexer parser codegen
// text -> token stream -> ast -> code?
// or treewalking interpreter idk
// values and types
// start with a tokenizer

// let's define a grammar
// assembly like?
// label
// procedure
// operation/function call
// define stack variable

// idk just have keywords for these
// types are just byte sizes + inout or maybe ref like rust tbh idk
// maybe have actual types
// should this align with actual processor architecture?
// or c?
// probably c
// proc <name> (<arg> out4, <arg> in4, ...) -> a, b(<var> 4) {<code>}
// label <name>
// op <op> <arg> ... [-> <label>, ...]
// var <name> 4

// then the lexer recognizes only a few types of tokens
// lpar rpar lbr rbr comma
// proc label op var
// name int

// i think i need semicolons too maybe to disambiguate
// no i don't there are keywords

// lex: &str -> iter<&str> ?
// -> 
// let's go for an iterator
// for a challenge

// how do i return multiple control flows
// also returning values
// i guess that's not hard it's out or inout parameters
// i would like nicer syntax but oh well

// yeah i think i need references
// otherwise some monomorphization or something
// nah bruh not doing that
// so &in &out &inout
// or maybe * instead of & whatever
// nah &
// only & parameters can be out for functions
// fn <name> (<arg> out4, <arg> in4, ...) {<code>}
// functions vs procedures
// functions are called with function calls
// procedures are inlined

// amp fn

// how do labels work
// first of all, goto
// then if
// then iter
// any op can have a jump in it
// or general control flow redirection
// op set a 4 -> a_label
// omitted defaults to the next statement
// some ops have multiple output control flows
// op if b -> true_case, false_case
// then procedures can be defined with multiple output control flows
// !!! only procedures, not functions !!!
// proc iter(it inout iterator 4) -> loop(value 4), end {...}

// eventually have types

use std::iter::Peekable;
use std::str::CharIndices;

// first define the token
#[derive(Debug)]
pub struct Token<'a> {
	source: &'a str, // do i need this ?
	text: &'a str, // the part of the source that corresponds to this token
	pub value: TokenValue<'a>,
}

impl<'a> Token<'a> {
	fn single_char(source: &'a str, start: usize, value: TokenValue<'a>) -> Token<'a> {
		Token::sized(source, start, 1, value)
	}

	fn sized(source: &'a str, start: usize, size: usize, value: TokenValue<'a>) -> Token<'a> {
		Token::new(source, start, start + size, value)
	}

	fn new(source: &'a str, start: usize, end: usize, value: TokenValue<'a>) -> Token<'a> {
		Token {
			source: source,
			text: &source[start..end],
			value: value,
		}
	}
}

// and the value
#[derive(Debug)]
pub enum TokenValue<'a> {
	LPAR,
	RPAR,
	LBR,
	RBR,
	COMMA,
	AMP,
	ARROW,
	PROC,
	FN,
	LABEL,
	OP,
	VAR,
	IN,
	OUT,
	NAME(&'a str), // techncally the &str is not necessary (it is always the same as text) // actually what if i want to 
	INT(i32),
}

impl<'a> Iterator for Lexer<'a> {
	type Item = Token<'a>;

	fn next(&mut self) -> Option<Token<'a>> {
		self.skip_whitespace();
		match self.iter.next() {
			None => None,
			Some((start, c)) => {
				// test the first character
				match c {
					'(' => Some(Token::single_char(&self.source, start, TokenValue::LPAR)),
					')' => Some(Token::single_char(&self.source, start, TokenValue::RPAR)),
					'{' => Some(Token::single_char(&self.source, start, TokenValue::LBR)),
					'}' => Some(Token::single_char(&self.source, start, TokenValue::RBR)),
					',' => Some(Token::single_char(&self.source, start, TokenValue::COMMA)),
					'&' => Some(Token::single_char(&self.source, start, TokenValue::AMP)),
					'-' => {
						if let Some((_, '>')) = self.iter.peek() {
							self.iter.next();
						} else {
							panic!("- always makes an arrow bro");
						}
						Some(Token::sized(&self.source, start, 2, TokenValue::ARROW))
					},
					_ if c.is_ascii_alphabetic() || c == '_' => {
						// take as many alphanumeric characters as possible
						let end = loop {
							match self.iter.peek() {
								None => break self.source.len(),
								Some((end, c)) => {
									if !(c.is_ascii_alphanumeric() || *c == '_') {
										break *end;
									} else {
										self.iter.next();
									}
								}
							}
						};
						Some(Token::new(&self.source, start, end, match &self.source[start..end] {
							"proc" => TokenValue::PROC,
							"fn" => TokenValue::FN,
							"label" => TokenValue::LABEL,
							"op" => TokenValue::OP,
							"var" => TokenValue::VAR,
							"in" => TokenValue::IN,
							"out" => TokenValue::OUT,
							text => TokenValue::NAME(text),
						}))
					 },
					_ if c.is_ascii_digit() => {
						// take as many digits as possible
						let end = loop {
							match self.iter.peek() {
								None => break self.source.len(),
								Some((end, c)) => {
									if !c.is_ascii_digit() {
										break *end;
									} else {
										self.iter.next();
									}
								}
							}
						};
						Some(Token::new(&self.source, start, end, TokenValue::INT(self.source[start..end].parse().expect("bro i thought it would be all digits"))))
					},
					_ => panic!("nooo unrecognized thing error"),
				}
			}
		}
	}
}

impl Lexer<'_> {
	fn skip_whitespace(&mut self) {
		while let Some((_, c)) = self.iter.peek() {
			if !c.is_ascii_whitespace() {
				break;
			}
			self.iter.next();
		}
	}
}

pub struct Lexer<'a> { // i'll add fields later
	source: &'a str,
	iter: Peekable<CharIndices<'a>>,
}

impl<'a> Lexer<'a> {
	pub fn new(source: &'a str) -> Self {
		Lexer {
			source: source,
			iter: source.char_indices().peekable(),
		}
	}
}