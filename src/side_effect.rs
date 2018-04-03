use super::StackValue;

/// A trait that can be constructed using `::default()`
/// that is used for dependency injection of IO-like operations
/// in the vm.
pub trait SideEffect: Default {
    /// Writes a line to stdout (or elsewhere)
    fn println(&mut self, value: StackValue);
    /// Reads a line from stdin (or elsewhere)
    fn read_line(&mut self) -> String;
    /// Sleeps a given number of ms
    fn sleep_ms(&mut self, duration: u64);
}

/// This is the default sideffect,
/// reading from STDIN, using `println!`,
/// and regular ol' sleep.
#[derive(Default)]
pub struct DefaultSideEffect;

impl SideEffect for DefaultSideEffect {

    fn read_line(&mut self) -> String {
        let mut input = String::new();
        ::std::io::stdin().read_line(&mut input).unwrap();
        input.trim().to_owned()
    }

    fn sleep_ms(&mut self, duration: u64) {
        use std::thread;
        use std::time;
        thread::sleep(time::Duration::from_millis(duration))
    }

    fn println(&mut self, value: StackValue) {
        println!("{}", value);
    }
}
