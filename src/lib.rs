//! Writing a simple stack based VM in rust based on https://csl.name/post/vm/.
//!
//! This won't have its own parser built in, but will operate on tokens...
//! The goal is to have this be something that actually lives on the stack if possible.

#![allow(dead_code)]

use std::str::FromStr;

enum StackOperationResult {
    Push(StackValue),
    SideEffect(()),
}

macro_rules! stack_operations {
    (match $stack:ident, $e:expr, $t:pat) => {
        match $stack.pop() {
            Some($t) => match $e {
                Push(val)=> $stack.push(val),
                _ => { }
            },
            None => panic!("No value to pop off the stack"),
            _ => panic!("Invalid argument type")
        }
    };

    (match $stack:ident, $e:expr, $t:pat, $($rest:pat),+) => {
        match $stack.pop() {
            Some($t) => stack_operations! { match $stack, $e, $($rest)+ },
            None => panic!("No value to pop off the stack"),
            _ => panic!("Invalid argument type")
        }
    };

    (
        $($t:ident $s:tt ($($type:pat),*) $e:expr),+
    ) => {

        #[derive(PartialEq, Debug)]
        pub enum StackOperation {
            $( $t, )+
        }

        impl FromStr for StackOperation {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                use StackOperation::*;
                match s {
                    $( stringify!($s) => Ok($t), )+
                    _ => Err(())
                }
            }
        }

        impl StackOperation {
            #[allow(unreachable_patterns)]
            pub fn dispatch(&self, stack: &mut Vec<StackValue>) {
                use StackValue::*;
                use StackOperationResult::*;
                match *self {
                    $(StackOperation::$t => {
                        stack_operations! { match stack, $e, $($type),+  }
                    },)+
                }
            }
        }
    }
}

stack_operations! {
    Plus + (Num(a), Num(b)) Push(Num(a + b)),
    Minus - (Num(a), Num(b)) Push(Num(b - a)),
    Prinln println (a @ _) SideEffect(println!("{}", a))
}

/// A value that can live on the stack.
#[derive(PartialEq, Debug)]
pub enum StackValue {
    Num(isize),
    Operation(StackOperation),
    String(String),
}

impl std::fmt::Display for StackValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use StackValue::*;
        match *self {
            Num(n) => n.fmt(f),
            String(ref s) => s.fmt(f),
            Operation(_) => write!(f, "<code>"),
        }
    }
}

/// ```
/// use simple_vm::{StackValue, StackOperation};
/// use std::str::FromStr;
///
/// // can parse numbers
/// let value = StackValue::from_str("1").unwrap();
///
/// // can parse strings (must be quoted)
/// let s = StackValue::from_str("\"hi\"").unwrap();
/// assert_eq!(StackValue::String("hi".to_owned()), s);
///
/// // can parse operations
/// let op = StackValue::from_str("+").unwrap();
/// assert_eq!(StackValue::Operation(StackOperation::Plus), op);
/// ```
impl FromStr for StackValue {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let len = s.len();
        if let Ok(n) = s.parse::<isize>() {
            return Ok(StackValue::Num(n));
        } else if let Ok(op) = StackOperation::from_str(s) {
            return Ok(StackValue::Operation(op));
        } if len > 1 && s.starts_with('"') && s.ends_with('"') {
            let substr = unsafe { s.get_unchecked(1..(len-1)) };
            return Ok(StackValue::String(substr.to_owned()));
        } else {
            return Err(());
        }
    }
}

pub type Stack = Vec<StackValue>;

pub struct Machine {
    stack: Stack,
    code: Vec<String>,
    instruction_ptr: usize,
}

impl Machine {
    pub fn new(code: Vec<String>) -> Self {
        Machine {
            stack: Stack::new(),
            code: code,
            instruction_ptr: 0
        }
    }

    pub fn run(&mut self) {
        while self.instruction_ptr < self.code.len() {
            let instruction = self.code.get(self.instruction_ptr).unwrap();
            let op = StackValue::from_str(instruction).unwrap();
            match op {
                StackValue::Operation(op) => op.dispatch(&mut self.stack),
                _ => self.stack.push(op)
            };
            self.instruction_ptr = self.instruction_ptr + 1;
        }
    }

}

pub fn run(code: Vec<String>) {
    let mut machine = Machine::new(code);
    machine.run();
}

#[test]
pub fn test_program() {
    let code = vec![ "1".to_owned(), "2".to_owned(), "+".to_owned() ];
    let mut machine = Machine::new(code);
    machine.run();
    assert_eq!(StackValue::Num(3), machine.stack[0]);
}
