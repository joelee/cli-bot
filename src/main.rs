use clap::Parser;

use cli_bot::{Cli, OutputStyler, run};

fn main() {
    let cli = Cli::parse();
    let output = OutputStyler::new(cli.color.clone());

    if let Err(error) = run(cli) {
        eprintln!("{} {error:#}", output.stderr_error("Error:"));
        std::process::exit(1);
    }
}
