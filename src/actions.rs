use crate::errors::Result;
use crate::stack::FileStack;

#[derive(Debug)]
pub enum Action {
    Push(String),
    List,
}

pub mod list;
pub mod push;

use Action::*;

pub fn invoke_action(action: Action) -> Result<()> {
    // TODO: look up the directory stack.
    let repo = git2::Repository::open(".")?;
    let mut stack = FileStack::new(&"./.git/BRANCH_STACK")?;

    match action {
        Push(ref branch_name) => push::push_branch(&repo, &mut stack, branch_name),
        List => list::list_branch_stack(&repo, &stack),
    }
}
