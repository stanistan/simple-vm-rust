/// This macro generates the `StackOperations` enum,
/// its implementations, how its parsed, and the modules/functions
/// for how each of the operations affects `Machine` state.
#[macro_export]
macro_rules! ops {

    // This means we can't evaluate the expression.
    (ERR $error_type:ident $t:pat, $e:expr) => {
        Err(StackError::$error_type {
            arg_pattern: stringify!($t).to_owned(),
            expr: stringify!($e).to_owned(),
        })
    };

    (POP $machine:ident) => {
        $machine.stack.pop()
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
        $machine.dispatch($e)
    };

    // The MATCH variants of this macro are so that we can recursively
    // generate code to pattern match on the arguments in the main
    // macro entrypoint...
    //
    // This is the penultimate when we are down to the last argument,
    // and need to pop one last value off of the stack.
    (MATCH $machine:ident, $e:expr, $t:pat) => {
        match ops!(POP $machine) {
            Some($t) => ops!(MATCH $machine, $e,),
            None => ops!(ERR EmptyStack $t, $e),
            _ => ops!(ERR PatternMismatch $t, $e),
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
        match ops!(POP $machine) {
            Some($t) => ops!(MATCH $machine, $e, $($rest),*),
            None => ops!(ERR EmptyStack $t, $e),
            _ => ops!(ERR PatternMismatch $t, $e),
        }
    };

    // This is the MAIN entry point for the macro.
    (
        $($(#[$attr:meta])* $t:ident $s:tt ($($type:pat),*) $e:expr,)+
    ) => {

        /// Generated enum of all the user-accessible primitive stack operations.
        ///
        /// This is generated by the `ops` macro.
        ///
        /// The enum name is the first arg to the macro.
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum StackOperation {
            $(
                $(#[$attr])* $t,
            )+
        }

        /// Each stack operation is able to be constructed from a string.
        /// This is the _second_ arg to the `ops` macro.
        impl FromStr for StackOperation {
            type Err = StackError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $( stringify!($s) => Ok(StackOperation::$t), )+
                    _ => Err(StackError::InvalidOperation {
                        name: s.into()
                    })
                }
            }
        }

        #[allow(non_snake_case)]
        pub mod impl_stack_operation {
            //!
            //! The `impl_stack_operation` module is generated by the `ops!` macro.
            //!
            //! Each of its submodules has an `execute` method which operates on a `Machine`.
            //!
            use super::*;
            $(pub mod $t {
                //! This is a generated module for a `StackOperation`.
                use super::*;
                /// Generated method by the `ops!` macro.
                #[allow(unreachable_patterns, unused_imports, unreachable_code)]
                pub fn execute(machine: &mut Machine) -> Result<bool, StackError> {
                    use StackValue::*;
                    use MachineOperation::*;
                    ops!(MATCH machine, $e, $($type),*)
                }
            })+
        }

        impl StackOperation {
            /// Dispatch a generated `StackOperation` variant to its relevant
            /// `impl_stack_operation::$OperationVariant::execute()` function.
            pub fn dispatch(&self, machine: &mut Machine) -> Result<bool,StackError> {
                match *self {
                    $(StackOperation::$t => impl_stack_operation::$t::execute(machine),)+
                }
            }
        }
    }

}
