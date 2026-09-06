// execute whatever compile.rs returns

// a value provides some sort of mutability
// no refcounting i think
// you can modify a value at any time
// so a Value has an Rc<actual value>
// ok sure
// hmm maybe box of refcell
// hold multiple references and
// ok probably rc refcell data

use crate::compile::{Compiled, OpName, Builtin, Arg, Callable, Type};
use std::cell::{RefCell, Ref};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Value {
	data: Rc<RefCell<Vec<u8>>>,
	valtype: VType,
}

#[derive(PartialEq, Clone, Debug)]
struct VType {
	size: i32,
}

impl From<&Type> for VType {
	fn from(t: &Type) -> VType {
		VType {size: t.size}
	}
}

impl Value {
	fn new(valtype: VType) -> Value {
		Value {data: Rc::new(RefCell::new((0..valtype.size).map(|_| 0).collect())), valtype: valtype}
	}

	pub fn new_i32(n: i32) -> Value {
		let v = Value::new(VType {size: 4});
		v.set(&n.to_le_bytes()[..]);
		v
	}

	fn set(&self, data: &[u8]) {
		self.data.replace(data.into());
	}

	fn get(&self) -> Ref<'_, [u8]> {
		Ref::map(self.data.borrow(), |v| &**v)
	}
}

// what is this
// i think a out control index + out var values
pub struct ExecResult;

pub fn execute(code: &Compiled, index: usize, args: &Vec<Value>) -> ExecResult {
	// execute a callable at the given index
	match &code.callables[index] {
		Callable::Proc(proc) => {
			assert!(args.len() == proc.argcount);
			let mut vars: Vec<Value> = args[..].into();
			vars.extend((proc.argcount..proc.vars.len()).map(|i| Value::new((&proc.vars[i].vartype).into())));
			// i love writing code with non type system managed invariants (vars.len() == proc.vars.len())
			let mut pc = 0;
			loop {
				let op = &proc.code[pc];
				// collect arguments
				let opargs: Vec<Value> = op.args.iter().map(|arg| {
					match arg {
						Arg::Var(i) => vars.get(*i).unwrap().clone(), // if this gives an error you passed something wrong or i wrote bad code
						Arg::Int(n) => Value::new_i32(*n),
					}
				}).collect();
				match op.name {
					OpName::UserDef(idx) => {
						// uhh like execute() it
						// maybe later divorce the language stack from rust's stack
						// collect arguments
						// assert types
						// execute()
						// wait do i need to return a result
						// or not?
						// yea probably
						execute(code, idx, &opargs)
					},
					OpName::Builtin(Builtin::RET) => {
						assert!(op.args.len() == 0 && op.targets.len() == 0);
						break ExecResult;
					},
					OpName::Builtin(Builtin::SET) => {
						match &opargs[..] {
							[Value {valtype: vt1, ..}, Value {valtype: vt2, ..}] if vt1 == vt2 => (),
							_ => panic!("no"),
						}
						match &op.targets[..] {
							[Target {vars: vars, ..}] => match &vars[..] {
								[] => (),
								_ => panic!("no"),
							},
							_ => panic!("no"),
						}
						opargs[0].set(&*opargs[1].get());
					},
				}
				pc += 1;
			}
		},
	}
}
