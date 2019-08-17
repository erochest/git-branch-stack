use std::error;
use std::fmt;
use std::convert::From;
use std::result;

use git2;

#[derive(Debug)]
pub enum BranchStackError {
    InvalidCommandError,
    ArgError(String),
    GitError(git2::Error),
    InvalidBranchName(String),
}

pub type Result<R> = result::Result<R, BranchStackError>;

use BranchStackError::*;

impl fmt::Display for BranchStackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidCommandError => write!(f, "invalid command"),
            ArgError(ref arg_name) => write!(f, "invalid argument value: {}", arg_name),
            GitError(ref err) => err.fmt(f),
            InvalidBranchName(ref name) => write!(f, "invalid branch name: {}", name),
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
        }
    }
}

impl From<git2::Error> for BranchStackError {
    fn from(err: git2::Error) -> Self {
        GitError(err)
    }
}
