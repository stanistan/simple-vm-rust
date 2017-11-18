extern crate failure;
#[macro_use] extern crate failure_derive;

use std::str::FromStr;

#[derive(Debug, Fail)]
pub enum StackError {
    /// Error condition for when we try to pop a value off
    /// the stack and it's empty for the given expression.
    #[fail(display="Cannot pop an empty stack, looking for {} in {}", arg_pattern, expr)]
    EmptyStack {
        arg_pattern: String,
        expr: String
    },
    /// Error condition for when the pattern provided for the
    /// value we've popped off the stack does not match the
    /// argument pattern provided for the expression.
    #[fail(display = "Pattern mismatch, looking for {} in {}", arg_pattern, expr)]
    PatternMismatch {
        arg_pattern: String,
        expr: String
    },
    /// Error condition when we could not parse the string.
    #[fail(display = "Could not parse \"{}\"", string)]
    InvalidString {
        string: String
    },
    /// Error condition for when a given string does not correspond to
    /// any defined operation.
    #[fail(display = "Invalid operation: {}", name)]
    InvalidOperation {
        name: String
    },
}

enum MachineOperation {
    Call(usize),
    Jump(usize),
    Push(Vec<StackValue>),
    Return,
    SideEffect(()),
    Stop,
}

macro_rules! debug {
    ($string:expr, $($rest:expr),*) => {{
        #[cfg(feature = "debug")]
        println!($string, $($rest),*);
    }}
}

macro_rules! stack_operations {

    (DEBUG $machine:ident $log:expr, $e:expr, $t:ty) => {{
        debug!("----------",);
        debug!("before:\t{:?}\t{:?}", $machine.stack, $machine.return_stack);
        debug!("op:\t{}", stringify!($log));
        let re: $t = $e;
        debug!("after:\t{:?}\t{:?}", $machine.stack, $machine.return_stack);
        re
    }};

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
        stack_operations!(
            DEBUG $machine $e,
            $machine.dispatch($e),
            Result<bool, StackError>
        )
    };

    (POP $machine:ident) => {
        stack_operations!(DEBUG $machine Pop, $machine.stack.pop(), Option<StackValue>)
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
        match stack_operations!(POP $machine) {
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
        match stack_operations!(POP $machine) {
            Some($t) => stack_operations!(MATCH $machine, $e, $($rest),*),
            None => stack_operations!(ERR EmptyStack $t, $e),
            _ => stack_operations!(ERR PatternMismatch $t, $e),
        }
    };

    // This is the MAIN entry point for the macro.
    (
        $($(#[$attr:meta])* $t:ident $s:tt ($($type:pat),*) $e:expr,)+
    ) => {

        /// Generated enum of all the user-accessible primitive stack operations.
        ///
        /// This is generated by the `stack_operations` macro.
        ///
        /// The enum name is the first arg to the macro.
        #[derive(Clone, PartialEq, Debug)]
        pub enum StackOperation {
            $(
                $(#[$attr])* $t,
            )+
        }

        /// Each stack operation is able to be constructed from a string.
        /// This is the _second_ arg to the `stack_operations` macro.
        impl FromStr for StackOperation {
            type Err = StackError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $( stringify!($s) => Ok(StackOperation::$t), )+
                    _ => Err(StackError::InvalidOperation {
                        name: s.to_owned()
                    })
                }
            }
        }

        impl StackOperation {
            #[allow(unreachable_patterns, unreachable_code)]
            pub fn dispatch(&self, machine: &mut Machine) -> Result<bool,StackError> {
                use StackValue::*;
                use MachineOperation::*;
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
    Plus + (Num(a), Num(b)) push(Num(a + b)),
    Minus - (Num(a), Num(b)) push(Num(b - a)),
    Multiply * (Num(a), Num(b)) push(Num(a * b)),
    Divide / (Num(a), Num(b)) push(Num(b / a)),
    ToInt cast_int (String(a)) push(Num(a.parse::<isize>().unwrap_or(0))),
    ToStr cast_str (a) push(String(format!("{}", a))),
    Println println (a) SideEffect(println!("{}", a)),
    Equals eq (a, b) push(Num(if a == b { 1 } else { 0 })),
    Mod % (Num(a), Num(b)) push(Num(b % a)),
    If if (f, t, Num(cond)) push(if cond == 0 { f } else { t }),
    Jump jmp (Num(a)) Jump(a as usize),
    Dup dup (val) push(vec![val.clone(), val]),
    SleepMS sleep_ms (Num(a)) SideEffect(util::sleep_ms(a as u64)),
    Exit exit (Num(exit_code)) SideEffect(util::exit(exit_code as i32)),
    Stop stop () Stop,
    Read read () push(String(util::read_line())),
    Over over (a, b) push(vec![b.clone(), a, b]),
    Call call (Num(a)) Call(a as usize),
    Return return () Return,
}

impl Into<Vec<StackValue>> for StackValue {
    fn into(self) -> Vec<StackValue> {
        vec![self]
    }
}

fn push<T: Into<Vec<StackValue>>>(val: T) -> MachineOperation {
    MachineOperation::Push(val.into())
}

mod util {

    pub fn exit(exit_code: i32) {
        ::std::process::exit(exit_code)
    }

    pub fn sleep_ms(duration: u64) {
        use ::std::thread;
        use ::std::time;
        thread::sleep(time::Duration::from_millis(duration))
    }

    pub fn read_line() -> String {
        let mut input = String::new();
        ::std::io::stdin().read_line(&mut input).unwrap();
        input.trim().to_owned()
    }

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

// TODO: Make this understand labels, so a person
// won't necessarily have to keep track of stuff
// like that on their own when doing subroutines using
// `call` and `return`.
impl FromStr for StackValue {
    type Err = StackError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use StackValue::*;
        let len = s.len();
        if let Ok(n) = s.parse::<isize>() {
            return Ok(Num(n));
        } else if let Ok(op) = StackOperation::from_str(s) {
            return Ok(Operation(op));
        } if len > 1 && s.starts_with('"') && s.ends_with('"') {
            let substr = unsafe { s.get_unchecked(1..(len-1)) };
            return Ok(String(substr.to_owned()));
        } else {
            return Err(StackError::InvalidString { string: s.to_owned() });
        }
    }
}

pub type Stack = Vec<StackValue>;

#[derive(Debug)]
pub struct Machine {
    pub stack: Stack,
    pub return_stack: Vec<usize>,
    code: Vec<String>,
    instruction_ptr: usize,
}

impl Machine {
    /// Create a new machine for the code.
    pub fn new(code: Vec<String>) -> Self {
        Machine {
            stack: Stack::new(),
            return_stack: Vec::new(),
            code: code,
            instruction_ptr: 0
        }
    }

    /// Move the instruction pointer to a given address.
    fn jump(&mut self, address: usize) {
        // TODO check for overflow here
        self.instruction_ptr = address;
    }

    /// Dispatch given the result from the stack operation,
    /// which gets consumed here.
    ///
    /// Returns true or false to indicate whether the `run` loop
    /// should continue.
    fn dispatch(&mut self, result: MachineOperation) -> Result<bool,StackError> {
        use MachineOperation::*;
        match result {
            Call(to) => {
                self.return_stack.push(self.instruction_ptr);
                self.jump(to);
            },
            Jump(to) => self.jump(to),
            Push(mut values) => self.stack.append(&mut values),
            Return => match self.return_stack.pop() {
                Some(jump_to) => self.jump(jump_to),
                _ => return stack_operations!(ERR EmptyStack Return, return)
            },
            SideEffect(_) => (),
            Stop => return Ok(false),
        }
        return Ok(true);
    }

    /// Runs the machine and either returns an Ok
    /// with an empty result, or a StackError
    /// on failure.
    pub fn run(&mut self) -> Result<(), StackError> {
        while self.instruction_ptr < self.code.len() {

            let current_instruction = self.instruction_ptr;
            let value = {
                let instruction = self.code.get(current_instruction).unwrap();
                StackValue::from_str(instruction).unwrap()
            };

            self.instruction_ptr = self.instruction_ptr + 1;

            if let StackValue::Operation(op) = value {
                if !op.dispatch(self)? {
                    break;
                }
            } else {
                stack_operations!(MATCH self, push(value),)?;
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
        test_call_return Num(4), [ "1" "1" "7" "call" "dup" "+" "stop" "+" "return" ],
    }

}
