extern crate failure;
#[macro_use]
extern crate failure_derive;

#[cfg(feature = "stats")]
extern crate heapsize;

#[cfg(feature = "stats")]
use heapsize::*;

use std::str::FromStr;
use std::collections::HashMap;

pub mod error;
use error::StackError;

#[macro_use]
pub mod stack_operations;

/// Primitive machine operations.
///
/// These are the operations that `StackOperation` is built on
/// and what they return to the machine.
///
/// Some of these are sideeffecty.
///
/// TODO, make the side-effects _injectable_.
pub enum MachineOperation {
    /// Adds the current `instruction_ptr` to the return stack and
    /// jumps to `usize`.
    Call(usize),
    /// Exits the process with the exit code.
    Exit(i32),
    /// Jumps to that instruction.
    Jump(usize),
    /// Does nothing
    NA,
    /// Appends many values to the stack.
    PushMany(Vec<StackValue>),
    /// Appends one value to the stack.
    Push(StackValue),
    /// Appends two values to the stack.
    PushTwo(StackValue, StackValue),
    /// Appends three values to the stack.
    PushThree(StackValue, StackValue, StackValue),
    /// Returns to the last thing added to the return stack.
    Return,
    /// Writes a value to stdout
    Println(StackValue),
    /// Sleeps :shrug:
    Sleep(u64),
    /// Stops execution of the Machine
    ///
    /// TODO: merge this with exit
    Stop,
}

ops! {
    Plus + (Num(a), Num(b)) Push(Num(a + b)),
    Minus - (Num(a), Num(b)) Push(Num(b - a)),
    Multiply * (Num(a), Num(b)) Push(Num(a * b)),
    Divide / (Num(a), Num(b)) Push(Num(b / a)),
    ToInt cast_int (String(a)) Push(Num(a.parse::<isize>().unwrap_or(0))),
    ToStr cast_str (a) Push(String(format!("{}", a))),
    Println println (a) Println(a),
    Equals == (a, b) Push(Bool(a == b)),
    Or or (Bool(a), Bool(b)) Push(Bool(a || b)),
    And and (Bool(a), Bool(b)) Push(Bool(a && b)),
    Not not (Bool(a)) Push(Bool(!a)),
    LessThan < (Num(a), Num(b)) Push(Bool(b < a)),
    LessThanOrEqualTo <= (Num(a), Num(b)) Push(Bool(b <= a)),
    GreaterHan > (Num(a), Num(b)) Push(Bool(b > a)),
    GreaterHanOrEqualto >= (Num(a), Num(b)) Push(Bool(b >= a)),
    Mod % (Num(a), Num(b)) Push(Num(b % a)),
    If if (f, t, Bool(cond)) Push(if cond { t } else { f }),
    Jump jmp (Num(a)) Jump(a as usize),
    Duplicate dup (val) PushTwo(val.clone(), val),
    Drop drop (_) NA,
    Rotate rot (a, b, c) PushThree(b, a, c),
    Swap swap (a, b) PushTwo(a, b),
    SleepMS sleep_ms (Num(a)) Sleep(a as u64),
    Exit exit (Num(exit_code)) Exit(exit_code as i32),
    Stop stop () Stop,
    Read read () Push(String(util::read_line())), //TODO should be a machine operation
    Over over (a, b) PushThree(b.clone(), a, b),
    Call call (Num(a)) Call(a as usize),
    Return return () Return,
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
        use std::thread;
        use std::time;
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
            Operation(ref op) => write!(f, "<op:{:?}>", op),
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
            Ok(StackValue::Bool(true))
        } else if s == "false" {
            Ok(StackValue::Bool(false))
        } else if let Ok(n) = s.parse::<isize>() {
            return Ok(Num(n));
        } else if let Ok(op) = StackOperation::from_str(s) {
            return Ok(Operation(op));
        } else if len > 1 && s.starts_with('"') && s.ends_with('"') {
            let substr = unsafe { s.get_unchecked(1..(len - 1)) };
            return Ok(String(substr.to_owned()));
        } else if len > 1 && s.ends_with(':') {
            let substr = unsafe { s.get_unchecked(0..(len - 1)) };
            return Ok(Label(substr.to_owned()));
        } else {
            return Ok(PossibleLabel(s.to_owned()));
        }
    }
}

/// If used with the feature=stats, this will be a struct populated
/// with runtime data about how the `Machine` is performing.
///
/// Otherwise it is empty!
#[cfg(feature = "stats")]
#[derive(Clone, Debug, Default)]
pub struct RunStats {
    pub args: Vec<StackValue>,
    pub instructions: usize,
    pub calls: usize,
    pub jumps: usize,
    pub returns: usize,
    pub max_stack_size: usize,
    pub max_stack_heap_size: usize,
    pub code_size: usize,
}

/// If used with the feature=stats, this will be a struct populated
/// with runtime data about how the `Machine` is performing.
///
/// Otherwise it is empty!
#[cfg(not(feature = "stats"))]
#[derive(Clone, Debug, Default)]
pub struct RunStats;

pub type Code = Vec<StackValue>;

#[derive(Debug)]
pub struct Machine {
    code: Code,
    instruction_ptr: usize,
    return_stack: Vec<usize>,
    stack: Vec<StackValue>,
    stats: RunStats,
}

macro_rules! stats {
    (inc $m:ident $field:ident) => {
        #[cfg(feature = "stats")]
        { $m.stats.$field += 1; }
    }
}

impl Machine {
    /// Create a new machine for the code.
    ///
    /// This runs through a `preprocess` step.
    pub fn new(code: Code) -> Result<Self, StackError> {
        let code = Self::preprocess(code)?;
        let len = code.len();
        Ok(Machine {
            code,
            instruction_ptr: 0,
            return_stack: Vec::new(),
            stack: Vec::with_capacity(len),
            stats: RunStats::default(),
        })
    }

    pub fn reset(&mut self) {
        self.instruction_ptr = 0;
        self.stats = RunStats::default();
        self.return_stack.drain(..);
        self.stack.drain(..);
    }

    pub fn stack(&self) -> Vec<StackValue> {
        self.stack.clone()
    }

    /// Takes `Code` as input and finds and replaces the
    /// labels with their actual positions.
    ///
    /// This will return `StackError` if there are labels used
    /// that have never been defined, or if there are labels
    /// that have been defined multiple times.
    pub fn preprocess(code: Code) -> Result<Code, StackError> {
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
        let replacements = {
            let mut labels_meta: HashMap<&str, (Vec<usize>, Vec<usize>)> = HashMap::new();
            let mut replacements = vec![];

            for (idx, value) in code.iter().enumerate() {
                if let StackValue::Label(ref s) = *value {
                    let entry = labels_meta.entry(s).or_insert((vec![], vec![]));
                    entry.0.push(idx + 1);
                } else if let StackValue::PossibleLabel(ref s) = *value {
                    let entry = labels_meta.entry(s).or_insert((vec![], vec![]));
                    entry.1.push(idx);
                }
            }

            for (key, val) in labels_meta {
                if val.0.len() > 1 {
                    return Err(StackError::MultipleLabelDefinitions {
                        label: (*key).into(),
                        locations: val.0.clone(),
                    });
                } else if val.0.is_empty() && !val.1.is_empty() {
                    return Err(StackError::UndefinedLabel {
                        label: (*key).into(),
                        times: val.1.len(),
                    });
                } else {
                    replacements.push((val.0[0], val.1));
                }
            }

            replacements
        };

        let mut code = code;
        for replacement in replacements {
            let place = replacement.0 as isize;
            for location in replacement.1 {
                code[location] = StackValue::Num(place);
            }
        }

        Ok(code)
    }

    /// Given an input string program, this returns a stack
    /// machine or an error based on not being to create/parse it.
    pub fn new_for_input(input: &str) -> Result<Self, StackError> {
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
    pub fn dispatch(&mut self, result: MachineOperation) -> Result<bool, StackError> {
        use MachineOperation::*;
        match result {
            Call(to) => {
                stats!(inc self calls);
                self.return_stack.push(self.instruction_ptr);
                self.jump(to);
            }
            Jump(to) => {
                stats!(inc self jumps);
                self.jump(to);
            }
            Push(val) => self.stack.push(val),
            PushTwo(v1, v2) => {
                self.stack.push(v1);
                self.stack.push(v2);
            },
            PushThree(v1, v2, v3) => {
                self.stack.push(v1);
                self.stack.push(v2);
                self.stack.push(v3);
            },
            PushMany(values) => self.stack_push(values),
            Return => match self.return_stack.pop() {
                Some(jump_to) => {
                    stats!(inc self returns);
                    self.jump(jump_to);
                }
                _ => return ops!(ERR EmptyStack Return, return),
            },
            Sleep(ms) => util::sleep_ms(ms),
            Exit(exit_code) => util::exit(exit_code),
            Println(val) => println!("{}", val),
            NA => (),
            Stop => return Ok(false),
        }
        Ok(true)
    }

    /// Set up the stats for this current run call.
    #[cfg(feature = "stats")]
    pub fn setup_stats(&mut self, args: Vec<StackValue>) {
        self.stats.code_size = self.code.heap_size_of_children();
        self.stats.args = args;
    }

    pub fn stack_push(&mut self, mut values: Vec<StackValue>) {
        self.stack.append(&mut values);
    }

    /// Steps forward once in the stack machine.
    ///
    /// If there is no further to go given the `instruction_ptr`,
    /// this will return `Ok(false)`, if there are further
    /// instructions to proceed with, `Ok(true)`, otherwise
    /// it will return an `Err(StackError)`.
    pub fn step(&mut self) -> Result<bool, StackError> {

        if self.instruction_ptr == self.code.len() {
            return Ok(false);
        }

        // We *first* borrow the value from the `code` we're running because
        // we might not actually need it (in case it's a label), otherwise we
        // clone it so that we can use it in the stack operations.
        //
        // If we use references here, the operation would end up having a mutable
        // reference to *this Machine* struct, which is a problem, since the `Machine`
        // owns the code that it operates on.
        let value: StackValue = {
            // this is safe because we did the bounds check above.
            let value: &StackValue = self.code.get(self.instruction_ptr).unwrap();
            self.instruction_ptr += 1;
            if let StackValue::Label(_) = *value {
                return Ok(true);
            } else {
                value.clone()
            }
        };

        // if it's an operation, we will dispatch it,
        // otherwise it's a regular value, push it
        // onto the stack.
        if let StackValue::Operation(op) = value {
            if !op.dispatch(self)? {
                return Ok(false);
            }
        } else {
            self.stack.push(value);
        }

        Ok(true)
    }

    /// Runs the machine with given arguments, and either a Result that might contain
    /// run stats if this was compiled with `features=stats`, otherwise an StackError
    /// if this failed for any reason.
    pub fn run(&mut self, args: Vec<StackValue>) -> Result<RunStats, StackError> {

        #[cfg(feature = "stats")]
        self.setup_stats(args.clone());

        self.stack_push(args);

        loop {
            match self.step() {
                Err(e) => return Err(e),
                Ok(false) => break,
                Ok(true) => {
                    #[cfg(feature = "stats")]
                    {
                        self.stats.instructions += 1;
                        let stack_size = self.stack.len();
                        if stack_size > self.stats.max_stack_size {
                            self.stats.max_stack_size = stack_size;
                            self.stats.max_stack_heap_size = self.stack.heap_size_of_children();
                        }
                    }
                },
            }
        }

        Ok(self.stats.clone())
    }
}

/// Given a `String` it should break this up into
/// a list of tokens that can be parsed into `StackValue`.
pub fn tokenize(input: &str) -> Result<Code, StackError> {
    struct ParserState {
        prev_is_escape: bool,
        ignore_til_eol: bool,
        token: String,
        tokens: Vec<StackValue>,
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
        token: String::new(),
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
            }
            '\\' => {
                if state.prev_is_escape {
                    state.prev_is_escape = false;
                    state.push_char(c);
                } else {
                    state.prev_is_escape = true;
                }
            }
            '#' => {
                if !state.token_is_string() {
                    state.ignore_til_eol = true;
                }
            }
            ' ' | '\n' | '\t' | '\r' => {
                if state.token_is_string() {
                    state.push_char(c);
                } else {
                    state.push_token()?;
                }
            }
            _ => {
                state.push_char(c);
            }
        }
    }
    state.push_token()?;
    Ok(state.tokens)
}

#[cfg(test)]
mod tests {

    use super::*;
    use StackValue::*;

    macro_rules! assert_tokens {
        ([ $($token:expr),* ], $test:expr) => {{
            let expected: Vec<super::StackValue> = vec![ $($token),* ];
            assert_eq!(expected, tokenize($test).unwrap());
        }}
    }

    #[test]
    fn test_tokenize() {
        assert_tokens!([], "# whatever man");
        assert_tokens!([], "# \"sup\" println read");
        assert_tokens!([], "      ");
        assert_tokens!([Num(0)], "0");
        assert_tokens!([Num(0), Num(1)], "0 1");
        assert_tokens!([String("hi".to_owned())], "\"hi\"");
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
                    machine.run(vec![]).unwrap();
                    assert_eq!($v, machine.stack[0]);
                }
            )+
        };
        (STACK $( $(#[$attr:meta])* $name:ident $v:expr, [ $code:expr ],)+) => {
            $(
                #[allow(unused_mut)]
                #[test]
                $(#[$attr])*
                fn $name() {
                    use Machine;
                    let code = super::tokenize($code).unwrap();
                    let mut machine = Machine::new(code).unwrap();
                    machine.run(vec![]).unwrap();
                    assert_eq!($v, machine.stack);
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

    test_run! { STACK
        test_addition_dup vec![Num(2), Num(2)], [ "1 1 + dup" ],
        test_addition_dup2 vec![Num(2), Num(2), Num(2)], [ "1 1 + dup dup" ],
    }


    #[test]
    fn test_stack_operations_are_tiny() {
        assert_eq!(1, ::std::mem::size_of::<StackOperation>());
    }

}
