/// # Errors
///
/// This sets up the library-specific error handling. It should cover all
/// errors that this library will need to deal with, including errors from
/// IO, git, and the command line.
use std::convert::From;
use std::error;
use std::fmt;
use std::io;
use std::result;

use git2;

/// The type enumerating all of the possible error states.
#[derive(Debug)]
pub enum BranchStackError {
    /// The user enters an unknown subcommand on the command line.
    InvalidCommandError,
    /// A command-line parameter value is invalid.
    ArgError(String),
    /// Errors from libgit2.
    GitError(git2::Error),
    /// The branch to push doesn't exist.
    InvalidBranchName(String),
    /// Not currently on a branch.
    NoCurrrentBranch,
    /// IO errors. Probably unable to read from or write to the stack.
    IoError(io::Error),
    /// Trying to pop off an empty stack.
    EmptyStack,
}

/// An alias to make working with these errors easier.
pub type Result<R> = result::Result<R, BranchStackError>;

use BranchStackError::*;

impl fmt::Display for BranchStackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidCommandError => write!(f, "invalid command"),
            ArgError(ref arg_name) => write!(f, "invalid argument value: {}", arg_name),
            GitError(ref err) => err.fmt(f),
            InvalidBranchName(ref name) => write!(f, "invalid branch name: {}", name),
            NoCurrrentBranch => write!(f, "no current branch"),
            IoError(ref err) => err.fmt(f),
            EmptyStack => write!(f, "empty stack"),
        }
    }
}

impl error::Error for BranchStackError {
    fn description(&self) -> &str {
        match self {
            InvalidCommandError => "invalid command",
            ArgError(_) => "invalid argument value",
            GitError(ref err) => err.description(),
            InvalidBranchName(_) => "invalid branch name",
            NoCurrrentBranch => "no current branch",
            IoError(ref err) => err.description(),
            EmptyStack => "empty stack",
        }
    }
}

impl From<git2::Error> for BranchStackError {
    fn from(err: git2::Error) -> Self {
        GitError(err)
    }
}

impl From<io::Error> for BranchStackError {
    fn from(err: io::Error) -> Self {
        IoError(err)
    }
}
