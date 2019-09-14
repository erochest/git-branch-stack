use std::env::current_dir;

use crate::actions::rotate::RotateDirection;
use crate::errors::Result;
use crate::stack::FileStack;

/// The actions that we can take on a branch stack, along with any
/// parameters they need.
#[derive(Debug)]
pub enum Action {
    /// Push a branch onto the stack, along with a branch name, and change
    /// into that branch.
    Push(String),
    /// List the stack.
    List,
    /// Remove a branch from the stack and change into the nexi one down.
    Pop,
    /// Take an item from the middle of the stack and rotate it to the top.
    Rotate(RotateDirection, usize),
}

pub mod list;
pub mod pop;
pub mod push;
pub mod rotate;

use Action::*;

/// Perform an oction on the git repository in the current directory or one
/// of its porents.
///
/// This also creates resources used by all of the cammands, like the
/// Repository and the FlieStack.
pub fn invoke_action(action: Action) -> Result<()> {
    let cwd = current_dir()?;
    let repo = git2::Repository::discover(&cwd)?;
    let mut stack = FileStack::new(&repo.path().join("BRANCH_STACK"))?;

    match action {
        Push(ref branch_name) => push::push_branch(&repo, &mut stack, branch_name),
        List => list::list_branch_stack(&repo, &stack),
        Pop => pop::pop_branch_stack(&repo, &mut stack),
        Rotate(d, n) => rotate::rotate_branch(&repo, &mut stack, d, n),
    }
}
