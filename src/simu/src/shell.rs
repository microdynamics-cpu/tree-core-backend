/// Command execution error
#[derive(Debug)]
pub enum ExecError {
    /// Empty command provided
    Empty,
    /// Exit from the shell loop
    Quit,
    /// Some arguments are missing
    MissingArgs,
    /// The provided command is unknown
    UnknownCommand(String),
    /// The history index is not valid
    InvalidHistory(usize),
    /// Other error that may have happen during command execution
    Other(Box<Error>),
}

use crate::ExecError::*;

impl fmt::Display for ExecError {
    fn fmt(&self, format: &mut fmt::Formatter) -> fmt::Result {
        return match *self {
            Empty => write!(format, "No command provided"),
            Quit => write!(format, "Quit"),
            UnknownCommand(ref cmd) => write!(format, "Unknown Command {}", cmd),
            InvalidHistory(i) => write!(format, "Invalid history entry {}", i),
            MissingArgs => write!(format, "Not enough arguments"),
            Other(ref e) => write!(format, "{}", e)
        };
    }
}