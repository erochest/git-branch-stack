/// # git-branch-stack
///
/// This allows you to juggle several git branches using an interface similar
/// to the shell commands `pushd`, `popd`, and `dirs`.
///
/// Pushing a branch saves your current branch to the stack and checks out
/// the new branch.
///
/// Popping a branch removes the current branch from the stack and checks
/// out the next branch down.
pub mod actions;
pub mod errors;
pub mod git;
pub mod stack;
