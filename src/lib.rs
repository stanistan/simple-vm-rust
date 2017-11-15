//! Writing a simple stack based VM in rust based on https://csl.name/post/vm/.
//!
//! This won't have its own parser built in, but will operate on tokens...
//! The goal is to have this be something that actually lives on the stack if possible.

#![allow(dead_code)]

use std::str::FromStr;

enum StackOperationResult {
    Jump(usize),
    Append(Vec<StackValue>),
    Push(StackValue),
    SideEffect(()),
}

macro_rules! stack_operations {

    (ERR empty_stack $t:pat, $e:expr) => {
        panic!("No value to pop off the stack: {} in {}", stringify!($t), stringify!($e))
    };

    (ERR mismach $t:pat) => {
        panic!("Invalid argument type on the stack: {}", stringify!($t))
    };

    (MATCH $machine:ident, $e:expr,) => {
        match $e {
            Append(mut values) => $machine.stack.append(&mut values),
            Push(val) => $machine.stack.push(val),
            Jump(address) => $machine.jump(address),
            SideEffect(_) => ()
        }
    };

    (MATCH $machine:ident, $e:expr, $t:pat) => {
        match $machine.stack.pop() {
            Some($t) => stack_operations!(MATCH $machine, $e,),
            None => stack_operations!(ERR empty_stack $t, $e),
            _ => stack_operations!(ERR mismach $t),
        }
    };

    (MATCH $machine:ident, $e:expr, $t:pat, $($rest:pat),*) => {
        match $machine.stack.pop() {
            Some($t) => stack_operations!(MATCH $machine, $e, $($rest)*),
            None => stack_operations!(ERR empty_stack $t, $e),
            _ => stack_operations!(ERR mismach $t),
        }
    };

    (
        $($(#[$attr:meta])* $t:ident $s:tt ($($type:pat)*) $e:expr,)+
    ) => {

        #[derive(Clone, PartialEq, Debug)]
        pub enum StackOperation {
            $( $(#[$attr])* $t, )+
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
            #[allow(unreachable_patterns, unreachable_code)]
            pub fn dispatch(&self, machine: &mut Machine) {
                use StackValue::*;
                use StackOperationResult::*;
                match *self {
                    $(StackOperation::$t => {
                        stack_operations!(MATCH machine, $e, $($type),*)
                    },)+
                }
            }
        }
    }
}

stack_operations! {
    Plus + (Num(a) Num(b)) Push(Num(a + b)),
    Minus - (Num(a) Num(b)) Push(Num(a - b)),
    Multiply * (Num(a) Num(b)) Push(Num(a * b)),
    Divide / (Num(a) Num(b)) Push(Num(a / b)),
    ToInt cast_int (String(a)) Push(Num(a.parse::<isize>().unwrap_or(0))),
    ToStr cast_str (a @ _) Push(String(format!("{}", a))),
    Println println (a @ _) SideEffect(println!("{}", a)),
    Jump jmp (Num(a)) Jump(a as usize),
    Dup dup (any @ _) Append(vec![any.clone(), any]),
    SleepMS sleep_ms (Num(a)) SideEffect(sleep_ms(a as u64)),
    Halt halt (Num(exit_code)) SideEffect(exit(exit_code as i32)),
    Read read () Push(String(read_line())),
}

fn exit(exit_code: i32) {
    std::process::exit(exit_code)
}

fn sleep_ms(duration: u64) {
    std::thread::sleep(std::time::Duration::from_millis(duration))
}

fn read_line() -> String {
    let mut input = std::string::String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_owned()
}

/// A value that can live on the stack.
#[derive(Clone, PartialEq, Debug)]
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
    pub stack: Stack,
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

    pub fn jump(&mut self, address: usize) {
        self.instruction_ptr = address;
    }

    pub fn run(&mut self) {
        while self.instruction_ptr < self.code.len() {
            let op = {
                let instruction = self.code.get(self.instruction_ptr).unwrap();
                StackValue::from_str(instruction).unwrap()
            };
            self.instruction_ptr = self.instruction_ptr + 1;
            match op {
                StackValue::Operation(op) => op.dispatch(self),
                _ => self.stack.push(op)
            };
        }
    }

}

pub fn run(code: Vec<String>) {
    let mut machine = Machine::new(code);
    machine.run();
}

#[cfg(test)]
mod test {

    use Machine;

    macro_rules! test_run {
        ($($name:ident $v:expr, [ $($code:expr)+ ],)+) => {
            $(
                #[test]
                fn $name() {
                    use StackValue::*;
                    let mut code = vec![];
                    $( code.push($code.to_owned()); )+
                    let mut machine = Machine::new(code);
                    machine.run();
                    assert_eq!($v, machine.stack[0]);
                }
            )+
        };
    }

    test_run! {
        test_addition Num(3), [ "1" "2" "+" ],
        test_cast_to_int Num(1), [ "\"1\"" "cast_int" ],
        test_cast_to_int_defaults_to_zero Num(0), [ "\"asdf\"" "cast_int" ],
        test_cast_to_str String("1".to_owned()), [ "1" "cast_str" ],
        test_cast_to_backwards Num(1), [ "1" "cast_str" "cast_int" ],
        test_dup Num(4), [ "1" "dup" "+" "dup" "+"],
    }

}
