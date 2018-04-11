extern crate failure;
#[macro_use] extern crate failure_derive;

use std::collections::HashMap;
use std::str::FromStr;

pub mod error;
pub mod side_effect;

use error::StackError;
pub use side_effect::*;

#[macro_use]
pub mod stack_operations;

/// Primitive machine operations.
///
/// These are the operations that `StackOperation` is built on
/// and what they return to the machine.
#[derive(Debug)]
pub enum MachineOperation {
    /// Adds the current `instruction_ptr` to the return stack and
    /// jumps to `usize`.
    Call(usize),
    /// Jumps to that instruction.
    Jump(usize),
    /// Does nothing
    NA,
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
    /// Reads from stdin
    ReadLn,
    /// Sleeps :shrug:
    Sleep(u64),
    /// Stops execution of the Machine with an exit code.
    Stop(i32),
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
    Exit exit (Num(exit_code)) Stop(exit_code as i32),
    Stop stop () Stop(0),
    Read read () ReadLn,
    Over over (a, b) PushThree(b.clone(), a, b),
    Call call (Num(a)) Call(a as usize),
    Return return () Return,
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
            String(ref s) => write!(f, "{}", s),
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

pub type Code = Vec<StackValue>;

/// Contains the exit code of the vm program.
pub struct RunResult {
    pub exit_code: i32,
}

#[derive(Copy, Clone)]
pub enum StepResult {
    Continue,
    Stop(i32),
}

/// This is the Stack Machine.
///
/// It is generic to a `SideEffect` which is used as
/// a zero-size memory dependency (at rust runtime),
/// but can be injected to test reading to and writing
/// from stdout.
#[derive(Debug)]
pub struct Machine<E>
where
    E: SideEffect,
{
    effect: E,
    pub code: Code,
    step: bool,
    instruction_ptr: usize,
    return_stack: Vec<usize>,
    stack: Vec<StackValue>,
}

impl<E: SideEffect> Machine<E> {
    /// Create a new machine for the code.
    ///
    /// This runs through a `preprocess` step.
    pub fn new(code: Code) -> Result<Self, StackError> {
        let code = Self::preprocess(code)?;
        let len = code.len();
        Ok(Machine {
            effect: E::default(),
            code,
            step: false,
            instruction_ptr: 0,
            return_stack: Vec::new(),
            stack: Vec::with_capacity(len),
        })
    }

    pub fn enable_step(&mut self) {
        self.step = true;
    }

    pub fn reset(&mut self) {
        self.instruction_ptr = 0;
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

    /// Move the instruction pointer to a given address.
    fn jump(&mut self, address: usize) {
        // TODO check for overflow here
        self.instruction_ptr = address;
    }

    /// Dispatch given the result from the stack operation, which gets consumed here.
    ///
    /// Returns an Error or a StepResult indicating how this loop should continue.
    pub fn dispatch(&mut self, op: MachineOperation) -> Result<StepResult, StackError> {

        use MachineOperation::*;

        if self.step {
            macro_rules! debug_step {
                ($($e:expr),*) => {
                    $(self.effect.println(StackValue::String(format!("{}:\t{:?}", stringify!($e), $e)));)*
                };
            }
            debug_step![op, self.code[self.instruction_ptr - 1], self.stack, self.return_stack];
            self.effect.println(StackValue::String(String::from("...")));
            self.effect.read_line();
        }

        match op {
            Call(to) => {
                self.return_stack.push(self.instruction_ptr);
                self.jump(to);
            }
            Jump(to) => {
                self.jump(to);
            }
            Push(val) => self.stack.push(val),
            PushTwo(v1, v2) => {
                self.stack.push(v1);
                self.stack.push(v2);
            }
            PushThree(v1, v2, v3) => {
                self.stack.push(v1);
                self.stack.push(v2);
                self.stack.push(v3);
            }
            Return => match self.return_stack.pop() {
                Some(jump_to) => {
                    self.jump(jump_to);
                }
                _ => return ops!(ERR EmptyStack Return, return),
            },
            Sleep(ms) => self.effect.sleep_ms(ms),
            Println(val) => self.effect.println(val),
            ReadLn => self.stack.push(StackValue::String(self.effect.read_line())),
            NA => (),
            Stop(code) => return Ok(StepResult::Stop(code)),
        }
        Ok(StepResult::Continue)
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
    pub fn step(&mut self) -> Result<StepResult, StackError> {
        if self.instruction_ptr == self.code.len() {
            return Ok(StepResult::Stop(0));
        }

        // We *first* borrow the value from the `code` we're running because
        // we might not actually need it (in case it's a label), otherwise we
        // clone it so that we can use it in the stack operations.
        //
        // If we use references here, the operation would end up having a mutable
        // reference to *this Machine* struct, which is a problem, since the `Machine`
        // owns the code that it operates on.
        let value: StackValue = {
            let value: &StackValue = &self.code[self.instruction_ptr];
            self.instruction_ptr += 1;
            if let StackValue::Label(_) = *value {
                return Ok(StepResult::Continue);
            } else {
                value.clone()
            }
        };

        // if it's an operation, we will dispatch it,
        // otherwise it's a regular value, push it
        // onto the stack.
        if let StackValue::Operation(op) = value {
            return op.dispatch(self);
        } else {
            self.stack.push(value);
            return Ok(StepResult::Continue);
        }
    }

    /// Runs the machine with given arguments,
    pub fn run(&mut self, args: Vec<StackValue>) -> Result<RunResult, StackError> {

        self.stack_push(args);

        loop {
            match self.step() {
                Err(e) => return Err(e),
                Ok(StepResult::Stop(exit_code)) => {
                    return Ok(RunResult {
                        exit_code,
                    })
                }
                Ok(StepResult::Continue) => {},
            }
        }
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
                self.token.clear();
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
                } else {
                    state.push_char(c);
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
        assert_tokens!([String("#".to_owned())], "\"#\"");
        assert_tokens!([Num(0)], "0");
        assert_tokens!([Num(0), Num(1)], "0 1");
        assert_tokens!([String("hi".to_owned())], "\"hi\"");
    }

    fn full_stack<E: SideEffect>(machine: Machine<E>) -> Vec<StackValue> {
        machine.stack
    }

    fn first_stack<E: SideEffect>(machine: Machine<E>) -> StackValue {
        machine.stack[0].clone()
    }

    fn effect<E: SideEffect>(machine: Machine<E>) -> E {
        machine.effect
    }

    #[derive(Debug, PartialEq, Eq)]
    struct NoIOEffect {
        line: std::string::String,
        output: Vec<std::string::String>,
        slept: Vec<u64>,
    }

    impl Default for NoIOEffect {
        fn default() -> NoIOEffect {
            NoIOEffect {
                line: "10".to_owned(), // we use this for testing
                output: vec![],
                slept: vec![],
            }
        }
    }

    impl SideEffect for NoIOEffect {
        fn read_line(&mut self) -> std::string::String {
            self.line.clone()
        }
        fn sleep_ms(&mut self, duration: u64) {
            self.slept.push(duration)
        }
        fn println(&mut self, value: StackValue) {
            self.output.push(format!("{}", value))
        }
    }

    macro_rules! effect {
        ($( $field:ident: $type:expr, )+) => {
            NoIOEffect {
                $( $field: $type, )+
                ..NoIOEffect::default()
            }
        }
    }

    macro_rules! test_run {
        ([ $f:ident ] $( $(#[$attr:meta])* $name:ident $c:expr, $v:expr, [ $code:expr ],)+) => {
            $(
                #[allow(unused_mut)]
                #[test]
                $(#[$attr])*
                fn $name() {
                    use Machine;
                    let code = super::tokenize($code).unwrap();
                    let mut machine = Machine::<NoIOEffect>::new(code).unwrap();
                    let output = machine.run(vec![]).unwrap();
                    assert_eq!($v, $f(machine));
                    assert_eq!($c, output.exit_code);
                }
            )+
        };
    }

    test_run! { [first_stack]

        test_addition 0, Num(3), [ "1 2 +" ],
        test_cast_to_int 0, Num(1), [ "\"1\" cast_int" ],
        test_cast_to_int_defaults_to_zero 0, Num(0), [ "\"asdf\" cast_int" ],
        test_cast_to_str 0, String("1".to_owned()), [ "1 cast_str" ],
        test_cast_to_backwards 0, Num(1), [ "1 cast_str cast_int" ],
        test_dup 0, Num(4), [ "1 dup + dup +"],
        test_if_true 0, Num(5), [ "true 5 10 if" ],
        test_if_false 0, Num(10), [ "false 5 10 if"  ],
        test_mod 0, Num(0), [ "4 2 %" ],
        test_dif 0, Num(2), [ "4 2 /" ],
        test_stop 0, Num(0), [ "0 stop 1 +" ],
        test_over 0, Num(4), [ "2 4 over / +" ],
        test_call_return 0, Num(4), [ "1 1 7 call dup + stop + return" ],
        test_label1 0, Num(0), [  "0 end jmp one: 1 + end: 0 +" ],
        test_label2 0, Num(1), [  "0 one jmp one: 1 + end: 0 +" ],
        test_swap 0, Num(2), [ "1 2 swap" ],
        test_drop 0, Num(2), [ "1 drop 2" ],
        test_readline 0, Bool(true), [ "10 read cast_int =="],
        test_rot1 0, Num(2), [ "1 2 3 rot" ],
        test_rot2 0, Num(5), [ "1 2 3 rot drop +" ],
        test_rot3 0, Num(3), [ "1 2 3 rot rot" ],
        test_and 0, Bool(true), [ "false not true and" ],
        test_or 0, Bool(true), [ "false true or" ],

        #[should_panic(expected = "EmptyStack")]
        test_pop 0, Num(0), ["cast_str"],

        #[should_panic(expected = "UndefinedLabel")]
        test_undefined_label 0, Num(0), [ "asdf" ],

        #[should_panic(expected = "MultipleLabelDefinitions")]
        test_multiple_label_definitions 0, Num(0), [ "a: a:" ],

    }

    fn empty_stack() -> Vec<StackValue> {
        vec![]
    }

    test_run! { [full_stack]
        test_addition_dup 0, vec![Num(2), Num(2)], [ "1 1 + dup" ],
        test_addition_dup2 0, vec![Num(2), Num(2), Num(2)], [ "1 1 + dup dup" ],
        test_exit_code_1 1, empty_stack(), [ "1 exit" ],
        test_exit_code_0 0, empty_stack(), [ "" ],
    }

    test_run! { [effect]
        test_sleep 0, effect! { slept: vec![10], }, [ "10 sleep_ms" ],
        test_subsequent_sleeps 0, effect! { slept: vec![1, 1, 1], }, [ "1 dup dup sleep_ms sleep_ms sleep_ms" ],
        test_writes 0, effect! { output: vec!["10".to_owned(), "10".to_owned()], }, [ "10 dup println cast_str println" ],
    }

    #[test]
    fn test_stack_operations_are_tiny() {
        assert_eq!(1, ::std::mem::size_of::<StackOperation>());
    }

}
