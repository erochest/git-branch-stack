use std::convert::From;
use std::error;
use std::fmt;
use std::io;
use std::result;

use git2;

#[derive(Debug)]
pub enum BranchStackError {
    InvalidCommandError,
    ArgError(String),
    GitError(git2::Error),
    InvalidBranchName(String),
    NoCurrrentBranch,
    IoError(io::Error),
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
            NoCurrrentBranch => write!(f, "no current branch"),
            IoError(ref err) => err.fmt(f),
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
