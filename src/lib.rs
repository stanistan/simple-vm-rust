extern crate failure;
#[macro_use] extern crate failure_derive;

#[cfg(feature = "stats")]
extern crate heapsize;

#[cfg(feature = "stats")]
use heapsize::*;

use std::str::FromStr;
use std::collections::HashMap;

#[derive(Debug, Fail)]
pub enum StackError {
    /// Error condition for when we try to pop a value off
    /// the stack and it's empty for the given expression.
    #[fail(display="Cannot pop an empty stack, looking for {} in {}", arg_pattern, expr)]
    EmptyStack {
        arg_pattern: String,
        expr: String
    },
    /// Error condition for when a given string does not correspond to
    /// any defined operation.
    #[fail(display = "Invalid operation: {}", name)]
    InvalidOperation {
        name: String
    },
    /// Error condition when we could not parse the string.
    #[fail(display = "Could not parse \"{}\"", string)]
    InvalidString {
        string: String
    },
    /// Error condition when a label is defined in multiple locations
    /// in the source.
    #[fail(display = "Label {} defined in locations: {:?}", label, locations)]
    MultipleLabelDefinitions {
        label: String,
        locations: Vec<usize>,
    },
    /// Error condition when the instruction pointer is out of bounds
    /// for the code provided to the machine.
    #[fail(display = "Out of bounds instruction pointer")]
    OutOfBounds,
    /// Error condition for when the pattern provided for the
    /// value we've popped off the stack does not match the
    /// argument pattern provided for the expression.
    #[fail(display = "Pattern mismatch, looking for {} in {}", arg_pattern, expr)]
    PatternMismatch {
        arg_pattern: String,
        expr: String
    },

    #[fail(display = "Program referes to undefined \"{}\" {} time(s)", label, times)]
    UndefinedLabel {
        label: String,
        times: usize
    },
}

/// Primitive machine operations.
///
/// These are the operations that StackOperation is built on
/// and what they return to the machine.
enum MachineOperation {
    Call(usize),
    Jump(usize),
    Push(Vec<StackValue>),
    Return,
    SideEffect(()),
    Stop,
}

/// If compiled with --features=debug, this will print
/// debugging information to stdout.
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

    (POP $machine:ident) => {
        stack_operations!(DEBUG $machine Pop, $machine.stack.pop(), Option<StackValue>)
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
    Equals == (a, b) push(Bool(a == b)),
    Or or (Bool(a), Bool(b)) push(Bool(a || b)),
    And and (Bool(a), Bool(b)) push(Bool(a && b)),
    Not not (Bool(a)) push(Bool(!a)),
    LessThan < (Num(a), Num(b)) push(Bool(b < a)),
    LessThanOrEqualTo <= (Num(a), Num(b)) push(Bool(b <= a)),
    GreaterHan > (Num(a), Num(b)) push(Bool(b > a)),
    GreaterHanOrEqualto >= (Num(a), Num(b)) push(Bool(b >= a)),
    Mod % (Num(a), Num(b)) push(Num(b % a)),
    If if (f, t, Bool(cond)) push(if cond { t } else { f }),
    Jump jmp (Num(a)) Jump(a as usize),
    Duplicate dup (val) push(vec![val.clone(), val]),
    Drop drop (_) SideEffect(()),
    Rotate rot (a, b, c) push(vec![b, a, c]),
    Swap swap (a, b) Push(vec![a, b]),
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

#[cfg(feature = "stats")]
impl HeapSizeOf for StackValue {
    fn heap_size_of_children(&self) -> usize {
        use StackValue::*;
        match *self {
            Bool(ref b) => b.heap_size_of_children(),
            Num(ref n) => n.heap_size_of_children(),
            Label(ref s) => s.heap_size_of_children(),
            Operation(ref o) => o.heap_size_of_children(),
            String(ref s) => s.heap_size_of_children(),
            PossibleLabel(ref s) => s.heap_size_of_children(),
        }
    }
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
    Bool(bool),
    Num(isize),
    Label(String),
    Operation(StackOperation),
    String(String),
    PossibleLabel(String),
}

impl std::fmt::Display for StackValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use StackValue::*;
        match *self {
            Bool(b) => b.fmt(f),
            Num(n) => n.fmt(f),
            Label(ref n) => write!(f, "{}:", n),
            String(ref s) => write!(f, "\"{}\"", s),
            Operation(_) => write!(f, "<code>"),
            PossibleLabel(ref s) => s.fmt(f),
        }
    }
}

impl FromStr for StackValue {
    type Err = StackError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use StackValue::*;
        let len = s.len();
        if s == "true" {
            return Ok(StackValue::Bool(true));
        } else if s == "false" {
            return Ok(StackValue::Bool(false));
        } else if let Ok(n) = s.parse::<isize>() {
            return Ok(Num(n));
        } else if let Ok(op) = StackOperation::from_str(s) {
            return Ok(Operation(op));
        } if len > 1 && s.starts_with('"') && s.ends_with('"') {
            let substr = unsafe { s.get_unchecked(1..(len-1)) };
            return Ok(String(substr.to_owned()));
        } else if len > 1 && s.ends_with(':') {
            let substr = unsafe { s.get_unchecked(0..(len-1)) };
            return Ok(Label(substr.to_owned()));
        } else {
            return Ok(PossibleLabel(s.to_owned()));
        }
    }
}

#[derive(Debug, Default)]
pub struct RunStats {
    pub instructions: usize,
    pub calls: usize,
    pub jumps: usize,
    pub returns: usize,
    pub max_stack_size: usize,
    pub code_size: usize,
}

#[derive(Debug)]
pub struct Machine {
    code: Vec<StackValue>,
    instruction_ptr: usize,
    return_stack: Vec<usize>,
    stack: Vec<StackValue>,
    stats: Option<RunStats>,
}

macro_rules! stats {
    (inc $m:ident $field:ident) => {
        #[cfg(feature = "stats")]
        {
            if let Some(ref mut stats) = $m.stats {
                stats.$field += 1;
            }
        }
    }
}

impl Machine {
    /// Create a new machine for the code.
    pub fn new(code: Vec<StackValue>) -> Result<Self, StackError> {
        let code = Machine::preprocess(code)?;
        Ok(Machine {
            code: code,
            instruction_ptr: 0,
            return_stack: Vec::new(),
            stack: Vec::new(),
            stats: None,
        })
    }

    fn preprocess(code: Vec<StackValue>) -> Result<Vec<StackValue>, StackError> {
        // The stack machine itself would know the labels
        // so we should know _before_ we run the code
        // whether or not there are malformed instructions.
        //
        // It's possible that a label is defined 2x which is a problem,
        // and it's possible that we have refer to invalid labels
        // in the program execution.
        //
        // The hashmap is keyed on the label name, and the value is tuple of:
        // 1. Do we have a location in code to point this to? How many?
        // 2. How many times is this label referenced?
        let mut code = code;
        let mut labels_meta: HashMap<String, (Vec<usize>, Vec<usize>)> = HashMap::new();

        for (idx, value) in code.iter().enumerate() {
            if let &StackValue::Label(ref s) = value {
                let entry = labels_meta.entry(s.clone()).or_insert((vec![], vec![]));
                entry.0.push(idx + 1);
            } else if let &StackValue::PossibleLabel(ref s) = value {
                let entry = labels_meta.entry(s.clone()).or_insert((vec![], vec![]));
                entry.1.push(idx);
            }
        }

        for (key, val) in labels_meta.iter() {
            if val.0.len() > 1 {
                return Err(StackError::MultipleLabelDefinitions{
                    label: key.clone(),
                    locations: val.0.clone(),
                });
            } else if val.0.is_empty() && !val.1.is_empty() {
                return Err(StackError::UndefinedLabel {
                    label: key.clone(),
                    times: val.1.len(),
                });
            } else {
                let location = val.0[0];
                for idx in val.1.iter() {
                    code[*idx] = StackValue::Num(location as isize);
                }
            }
        }

        Ok(code)
    }

    pub fn new_for_input(input: &str) -> Result<Self,StackError> {
        let code = tokenize(input)?;
        Self::new(code)
    }

    /// Move the instruction pointer to a given address.
    fn jump(&mut self, address: usize) {
        // TODO check for overflow here
        self.instruction_ptr = address;
    }

    /// Dispatch given the result from the stack operation, which gets consumed here.
    ///
    /// Returns true or false to indicate whether the `run` loop should continue.
    fn dispatch(&mut self, result: MachineOperation) -> Result<bool,StackError> {
        use MachineOperation::*;
        match result {
            Call(to) => {
                stats!(inc self calls);
                self.return_stack.push(self.instruction_ptr);
                self.jump(to);
            },
            Jump(to) => {
                stats!(inc self jumps);
                self.jump(to);
            }
            Push(mut values) => self.stack.append(&mut values),
            Return => match self.return_stack.pop() {
                Some(jump_to) => {
                    stats!(inc self returns);
                    self.jump(jump_to);
                },
                _ => return stack_operations!(ERR EmptyStack Return, return)
            },
            SideEffect(_) => (),
            Stop => return Ok(false),
        }
        return Ok(true);
    }

    /// Runs the machine and either returns an Ok with an empty result,
    /// or a StackError on failure.
    pub fn run(&mut self) -> Result<Option<RunStats>, StackError> {

        #[cfg(feature = "stats")]
        {
            let mut stats = RunStats::default();
            stats.code_size = self.code.heap_size_of_children();
            self.stats = Some(stats);
        }

        use StackValue::*;
        while self.instruction_ptr < self.code.len() {

            // attempt to construct a StackValue from the current position in code
            let value: StackValue = match self.code.get(self.instruction_ptr) {
                None => return Err(StackError::OutOfBounds),
                Some(instruction) => instruction.clone()
            };

            // move forward in instructions
            self.instruction_ptr = self.instruction_ptr + 1;

            // evalate the value, if it's a label we just skip,
            // if it's an operation, we dispatch it,
            // otherwise we push the value onto the stack.
            match value {
                Label(_) => (),
                Operation(op) => {
                    if !op.dispatch(self)? {
                        break;
                    }
                },
                _ => {
                    stack_operations!(MATCH self, push(value),)?;
                },
            };

            #[cfg(feature = "stats")]
            {
                if let Some(ref mut stats) = self.stats {
                    stats.instructions += 1;
                    let stack_size = self.stack.len();
                    if stack_size > stats.max_stack_size {
                        stats.max_stack_size = stack_size;
                    }
                }
            }

        }

        Ok(self.stats.take())
    }

}

/// Given a string it should break this up into
/// a list of tokens that can be parsed into StackValue.
pub fn tokenize(input: &str) -> Result<Vec<StackValue>, StackError> {

    struct ParserState {
        prev_is_escape: bool,
        ignore_til_eol: bool,
        token: String,
        tokens: Vec<StackValue>
    }

    impl ParserState {
        fn push_char(&mut self, c: char) {
            if !self.ignore_til_eol {
                self.token.push(c);
            }
        }
        fn push_token(&mut self) -> Result<(), StackError> {
            if !self.token.is_empty() {
                let value = StackValue::from_str(&self.token)?;
                self.tokens.push(value);
                self.token = String::new();
            }
            Ok(())
        }
        fn token_is_string(&mut self) -> bool {
            self.token.starts_with('"')
        }
    }

    let mut state = ParserState {
        prev_is_escape: false,
        ignore_til_eol: false,
        tokens: Vec::new(),
        token: String::new()
    };

    for c in input.chars() {

        if state.ignore_til_eol {
            if c == '\n' {
                state.ignore_til_eol = false;
            }
            continue;
        }

        match c {
            '"' => {
                state.push_char(c);
                if !state.prev_is_escape && state.token.len() > 1 {
                    state.push_token()?;
                } else {
                    state.prev_is_escape = false;
                }
            },
            '\\' => {
                if state.prev_is_escape {
                    state.prev_is_escape = false;
                    state.push_char(c);
                } else {
                    state.prev_is_escape = true;
                }
            },
            '#' => {
                if !state.token_is_string() {
                    state.ignore_til_eol = true;
                }
            },
            ' ' | '\n' | '\t' | '\r' => {
                if state.token_is_string() {
                    state.push_char(c);
                } else {
                    state.push_token()?;
                }
            },
            _ => {
                state.push_char(c);
            },
        }
    }
    state.push_token()?;
    return Ok(state.tokens);

}

#[cfg(test)]
mod tests {

    use StackValue::*;
    use super::tokenize;

    macro_rules! assert_tokens {
        ([ $($token:expr),* ], $test:expr) => {{
            let expected: Vec<super::StackValue> = vec![ $($token),* ];
            assert_eq!(expected, tokenize($test).unwrap());
        }}
    }

    #[test]
    fn test_tokenize() {
        assert_tokens!( [ ], "# whatever man");
        assert_tokens!( [ ], "# \"sup\" println read");
        assert_tokens!( [ ], "      ");
        assert_tokens!( [ Num(0) ], "0" );
        assert_tokens!( [ Num(0), Num(1) ], "0 1" );
        assert_tokens!( [ String("hi".to_owned()) ], "\"hi\"" );
    }

    macro_rules! test_run {
        ($( $(#[$attr:meta])* $name:ident $v:expr, [ $code:expr ],)+) => {
            $(
                #[allow(unused_mut)]
                #[test]
                $(#[$attr])*
                fn $name() {
                    use Machine;
                    let code = super::tokenize($code).unwrap();
                    let mut machine = Machine::new(code).unwrap();
                    machine.run().unwrap();
                    assert_eq!($v, machine.stack[0]);
                }
            )+
        };
    }

    test_run! {

        test_addition Num(3), [ "1 2 +" ],
        test_cast_to_int Num(1), [ "\"1\" cast_int" ],
        test_cast_to_int_defaults_to_zero Num(0), [ "\"asdf\" cast_int" ],
        test_cast_to_str String("1".to_owned()), [ "1 cast_str" ],
        test_cast_to_backwards Num(1), [ "1 cast_str cast_int" ],
        test_dup Num(4), [ "1 dup + dup +"],
        test_if_true Num(5), [ "true 5 10 if" ],
        test_if_false Num(10), [ "false 5 10 if"  ],
        test_mod Num(0), [ "4 2 %" ],
        test_dif Num(2), [ "4 2 /" ],
        test_stop Num(0), [ "0 stop 1 +" ],
        test_over Num(4), [ "2 4 over / +" ],
        test_call_return Num(4), [ "1 1 7 call dup + stop + return" ],
        test_label1 Num(0), [  "0 end jmp one: 1 + end: 0 +" ],
        test_label2 Num(1), [  "0 one jmp one: 1 + end: 0 +" ],
        test_swap Num(2), [ "1 2 swap" ],
        test_drop Num(2), [ "1 drop 2" ],
        test_rot1 Num(2), [ "1 2 3 rot" ],
        test_rot2 Num(5), [ "1 2 3 rot drop +" ],
        test_rot3 Num(3), [ "1 2 3 rot rot" ],
        test_and Bool(true), [ "false not true and" ],
        test_or Bool(true), [ "false true or" ],

        #[should_panic(expected = "EmptyStack")]
        test_pop Num(0), ["cast_str"],

        #[should_panic(expected = "UndefinedLabel")]
        test_undefined_label Num(0), [ "asdf" ],

        #[should_panic(expected = "MultipleLabelDefinitions")]
        test_multiple_label_definitions Num(0), [ "a: a:" ],
    }

}
