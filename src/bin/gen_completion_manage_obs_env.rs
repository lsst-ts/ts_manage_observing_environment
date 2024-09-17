use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;
use ts_observing_environment::manage_obs_env::ManageObsEnv;

fn main() {
    let mut command = ManageObsEnv::command();
    let bin_name = command.get_name().to_string();

    generate(Shell::Bash, &mut command, bin_name, &mut io::stdout())
}
