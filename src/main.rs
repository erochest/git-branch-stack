use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg, ArgMatches,
    SubCommand,
};

use git_branch_stack::actions::rotate::parse_rotation;
use git_branch_stack::actions::{invoke_action, Action};
use git_branch_stack::errors::{BranchStackError, Result};

/// The main entry-point. Not really interesting.
fn main() -> Result<()> {
    let action = parse_args()?;
    invoke_action(action)
}

/// Parse all of the command-line options into an `Action` that can be run.
fn parse_args() -> Result<Action> {
    let arg_matches = app_from_crate!()
        .about("Maintain a stack of branches for easy navigation.")
        .subcommand(
            SubCommand::with_name("push")
                .about("Pushes a new branch onto tho stack.")
                .arg(
                    Arg::with_name("branch")
                        .help(
                            "The name of the branch to switch to. A number \
                             like +1 or -1 rotates the stack until that \
                             number (starting at 0, or counting from the \
                             right for negative numbers) branch in on top.",
                        )
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(SubCommand::with_name("list").about("List the branches in the branch stack."))
        .subcommand(
            SubCommand::with_name("pop")
                .about("Remove the top of the stack and change to the next one down."),
        )
        .get_matches();

    if let Some(push_args) = arg_matches.subcommand_matches("push") {
        parse_push_args(push_args)
    } else if arg_matches.subcommand_matches("list").is_some() {
        Ok(Action::List)
    } else if arg_matches.subcommand_matches("pop").is_some() {
        Ok(Action::Pop)
    } else {
        Err(BranchStackError::InvalidCommandError)
    }
}

/// Parse command-line arguments into parameters for the `push` command.
fn parse_push_args<'a>(push_args: &ArgMatches<'a>) -> Result<Action> {
    push_args
        .value_of("branch")
        .map(|branch_name| {
            if let Some((dir, n)) = parse_rotation(branch_name) {
                Action::Rotate(dir, n)
            } else {
                Action::Push(branch_name.to_string())
            }
        })
        .ok_or_else(|| BranchStackError::ArgError(String::from("branch")))
}
