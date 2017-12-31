/// The possible error conditions.
#[derive(Debug, Fail)]
pub enum StackError {
    /// Error condition for when we try to pop a value off
    /// the stack and it's empty for the given expression.
    #[fail(display = "Cannot pop an empty stack, looking for {} in {}", arg_pattern, expr)]
    EmptyStack { arg_pattern: String, expr: String },
    /// Error condition for when a given string does not correspond to
    /// any defined operation.
    #[fail(display = "Invalid operation: {}", name)]
    InvalidOperation { name: String },
    /// Error condition when we could not parse the string.
    #[fail(display = "Could not parse \"{}\"", string)]
    InvalidString { string: String },
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
    PatternMismatch { arg_pattern: String, expr: String },

    #[fail(display = "Program referes to undefined \"{}\" {} time(s)", label, times)]
    UndefinedLabel { label: String, times: usize },
}
