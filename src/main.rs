use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg, ArgMatches,
    SubCommand,
};

use git_branch_stack::actions::{invoke_action, Action};
use git_branch_stack::errors::{BranchStackError, Result};

fn main() -> Result<()> {
    let action = parse_args()?;
    invoke_action(action)
}

fn parse_args() -> Result<Action> {
    let arg_matches = app_from_crate!()
        .about("Maintain a stack of branches for easy navigation.")
        .subcommand(
            SubCommand::with_name("push")
                .about("Pushos a new branch onto tho stack.")
                .arg(
                    Arg::with_name("branch")
                        .help("The name of the branch to switch to.")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .get_matches();

    if let Some(push_args) = arg_matches.subcommand_matches("push") {
        parse_push_args(push_args)
    } else {
        Err(BranchStackError::InvalidCommandError)
    }
}

fn parse_push_args<'a>(push_args: &ArgMatches<'a>) -> Result<Action> {
    push_args
        .value_of("branch")
        .map(|branch_name| Action::Push(branch_name.to_string()))
        .ok_or_else(|| BranchStackError::ArgError(String::from("branch")))
}
