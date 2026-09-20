use clap::Parser;

use cli_bot::{Cli, OutputStyler, exit_code, run};

fn main() {
    let cli = Cli::parse();
    let output = OutputStyler::new(cli.color.clone());

    if let Err(error) = run(cli) {
        eprintln!("{} {error:#}", output.stderr_error("Error:"));
        // A command that ran and failed lends the shell its own status.
        std::process::exit(exit_code(&error));
    }
}
