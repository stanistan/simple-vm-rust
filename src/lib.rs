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

//
// Each stack operation should potentially have a function pointer
// and types that it expects, it'd be nice to have a macro dsl
// for defining it...  Something like:
//
//  ```
//  stack_operations! {
//      Plus "+" (a: Num, b: Num) => stack.push(Num(a + b)),
//      Minus "-" (a: Num, b: Num) => stack.push(Num(a - b)),
//      Print "print" (a: Num|String) => print!("{}", a),
//  }
//  ```
//
// This should end up:
//
// 1. Generating the regular `enum` block containing all stack operations.
// 2. Generating the `from_str` block for all of the second part
// 3. Generate an impl block with a `dispatch` function
//
macro_rules! stack_operations {
    (match $stack:ident, $e:expr, $t:pat) => {
        match $stack.pop() {
            // TODO, make this something that dispatches on an enum
            // so that the thing doesn't need to blablabla
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

        // generate the enum,
        // these have no contained data,
        // so this is super simple!
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

impl StackOperation {
    /*
    pub fn dispatch(&self, stack: &mut Vec<StackValue>) {
        use StackOperation::*;
        use StackValue::*;
        match *self {
            Plus => {
                let num_1 = stack.pop().unwrap();
                let num_2 = stack.pop().unwrap();
                match (num_1, num_2) {
                    (Num(a), Num(b)) => stack.push(Num(a + b)),
                    _ => panic!("fuck")
                }
            },
            Minus => {
                let num_1 = stack.pop().unwrap();
                let num_2 = stack.pop().unwrap();
                match (num_1, num_2) {
                    (Num(a), Num(b)) => stack.push(Num(b - a)),
                    _ => panic!("fuck")
                }
            },
            Print => {
                print!("{}", stack.pop().unwrap());
            },
            Println => {
                println!("{}", stack.pop().unwrap());
            }
        }
    }
    */
}

/*
impl FromStr for StackOperation {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use StackOperation::*;
        match s {
            "+" => Ok(Plus),
            "-" => Ok(Minus),
            "print" => Ok(Print),
            "println" => Ok(Println),
            _ => Err(())
        }
    }
}*/

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
