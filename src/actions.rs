use std::env::current_dir;

use crate::errors::Result;
use crate::stack::FileStack;

#[derive(Debug)]
pub enum Action {
    Push(String),
    List,
    Pop,
}

pub mod list;
pub mod pop;
pub mod push;

use Action::*;

pub fn invoke_action(action: Action) -> Result<()> {
    let cwd = current_dir()?;
    let repo = git2::Repository::discover(&cwd)?;
    let mut stack = FileStack::new(&repo.path().join("BRANCH_STACK"))?;

    match action {
        Push(ref branch_name) => push::push_branch(&repo, &mut stack, branch_name),
        List => list::list_branch_stack(&repo, &stack),
        Pop => pop::pop_branch_stack(&repo, &mut stack),
    }
}
