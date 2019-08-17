use crate::errors::Result;

#[derive(Debug)]
pub enum Action {
    Push(String),
}

pub mod push;

use Action::*;

pub fn invoke_action(action: Action) -> Result<()> {
    // TODO: look up the directory stack.
    let repo = git2::Repository::open(".")?;
    match action {
        Push(ref branch_name) => push::push_branch(repo, branch_name),
    }
}
