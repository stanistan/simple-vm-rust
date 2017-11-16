//! Writing a simple stack based VM in rust based on https://csl.name/post/vm/.
//!
//! This won't have its own parser built in, but will operate on tokens...
//! The goal is to have this be something that actually lives on the stack if possible.

#![allow(dead_code)]

use std::str::FromStr;

enum StackOperationResult {
    Append(Vec<StackValue>),
    Jump(usize),
    Push(StackValue),
    SideEffect(()),
    Stop,
}

impl StackOperationResult {
    fn dispatch(self, machine: &mut Machine) -> bool {
        use StackOperationResult::*;
        match self {
            Append(mut values) => machine.stack.append(&mut values),
            Jump(address) => machine.jump(address),
            Push(val) => machine.stack.push(val),
            SideEffect(_) => (),
            Stop => return false,
        }
        return true;
    }
}

#[derive(Debug)]
pub enum StackError {
    /// Error condition for when we try to pop a value off
    /// the stack and it's empty for the given expression.
    EmptyStack {
        arg_pattern: String,
        expr: String
    },
    /// Error condition for when the pattern provided for the
    /// value we've popped off the stack does not match the
    /// argument pattern provided for the expression.
    PatternMismatch {
        arg_pattern: String,
        expr: String
    },
}

macro_rules! stack_operations {

    // This means we can't evaluate the expression.
    (ERR $error_type:ident $t:pat, $e:expr) => {
        Err(StackError::$error_type {
            arg_pattern: stringify!($t).to_owned(),
            expr: stringify!($e).to_owned(),
        })
    };

    // The MATCH variants of this macro are so that we can recursively
    // generate code to pattern match on the arguments in the main
    // macro entrypoint...
    //
    // This is the LEAF recursive match pattern for when we have gone through
    // every single one of the potential variants and everything has succeeded.
    //
    // The expression is evaluated and given the result type,
    // we do something with the stack.
    (MATCH $machine:ident, $e:expr,) => {
        Ok($e.dispatch($machine))
    };

    // The MATCH variants of this macro are so that we can recursively
    // generate code to pattern match on the arguments in the main
    // macro entrypoint...
    //
    // This is the penultimate when we are down to the last argument,
    // and need to pop one last value off of the stack.
    //
    // NOTE that the trailing comma in the Some branch is super important.
    (MATCH $machine:ident, $e:expr, $t:pat) => {
        match $machine.stack.pop() {
            Some($t) => stack_operations!(MATCH $machine, $e,),
            None => stack_operations!(ERR EmptyStack $t, $e),
            _ => stack_operations!(ERR PatternMismatch $t, $e),
        }
    };

    // The MATCH variants of this macro are so that we can recursively
    // generate code to pattern match on the arguments in the main
    // macro entrypoint...
    //
    // This is the main recursion point where we start with a pattern
    // to pop an argument from the stack and match it, and if it succeeds
    // continue to recurse with the $rest.
    (MATCH $machine:ident, $e:expr, $t:pat, $($rest:pat),*) => {
        match $machine.stack.pop() {
            Some($t) => stack_operations!(MATCH $machine, $e, $($rest),*),
            None => stack_operations!(ERR EmptyStack $t, $e),
            _ => stack_operations!(ERR PatternMismatch $t, $e),
        }
    };

    // This is the MAIN entry point for the macro.
    //
    // The form of arguments this macro takes is something like:
    //
    // ```
    //  /// Variant/function documentation (this is optional)
    //  EnumVariantName
    //  ident_of_what_the_vm_understands
    //  (ValueVariantPattern(var))
    //  StackOperationResultVariant(operateOnVal(var)),
    // ```
    //
    // The trailing comma is required.
    (
        $($(#[$attr:meta])* $t:ident $s:tt ($($type:pat),*) $e:expr,)+
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
            pub fn dispatch(&self, machine: &mut Machine) -> Result<bool,StackError> {
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
    Plus + (Num(a), Num(b)) Push(Num(a + b)),
    Minus - (Num(a), Num(b)) Push(Num(b - a)),
    Multiply * (Num(a), Num(b)) Push(Num(a * b)),
    Divide / (Num(a), Num(b)) Push(Num(b / a)),
    ToInt cast_int (String(a)) Push(Num(a.parse::<isize>().unwrap_or(0))),
    ToStr cast_str (a) Push(String(format!("{}", a))),
    Println println (a) SideEffect(println!("{}", a)),
    Equals eq (a, b) Push(Num(if a == b { 1 } else { 0 })),
    Mod % (Num(a), Num(b)) Push(Num(b % a)),
    If if (f, t, Num(cond)) Push(if cond == 0 { f } else { t }),
    Jump jmp (Num(a)) Jump(a as usize),
    Dup dup (any) Append(vec![any.clone(), any]),
    SleepMS sleep_ms (Num(a)) SideEffect(sleep_ms(a as u64)),
    Exit exit (Num(exit_code)) SideEffect(exit(exit_code as i32)),
    Stop stop () Stop,
    Read read () Push(String(read_line())),
    Over over (a, b) Append(vec![b.clone(), a, b]),
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

    pub fn run(&mut self) -> Result<(), StackError> {
        while self.instruction_ptr < self.code.len() {

            let value = {
                let instruction = self.code.get(self.instruction_ptr).unwrap();
                StackValue::from_str(instruction).unwrap()
            };

            self.instruction_ptr = self.instruction_ptr + 1;

            if let StackValue::Operation(op) = value {
                if !op.dispatch(self)? {
                    break;
                }
            } else {
                self.stack.push(value)
            }
        }

        Ok(())
    }

}

pub fn run(code: Vec<String>) -> Result<(), StackError> {
    let mut machine = Machine::new(code);
    machine.run()
}

#[cfg(test)]
mod test {

    use Machine;

    macro_rules! test_run {
        ($( $(#[$attr:meta])* $name:ident $v:expr, [ $($code:expr)* ],)+) => {
            $(
                #[allow(unused_mut)]
                #[test]
                $(#[$attr])*
                fn $name() {
                    use StackValue::*;
                    let mut code = vec![];
                    $( code.push($code.to_owned()); )*
                    let mut machine = Machine::new(code);
                    machine.run().unwrap();
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
        test_if_true Num(5), [ "1" "5" "10" "if" ],
        test_if_false Num(10), [ "0" "5" "10" "if"  ],
        test_mod Num(0), [ "4" "2" "%" ],
        test_dif Num(2), [ "4" "2" "/" ],
        test_stop Num(0), [ "0" "stop" "1" "+" ],
        test_over Num(4), [ "2" "4" "over" "/" "+" ],
        #[should_panic(expected = "EmptyStack")] test_pop Num(0), ["cast_str"],
    }


}
