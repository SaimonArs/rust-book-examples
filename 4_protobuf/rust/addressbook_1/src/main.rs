use std::process;

use addressbook_1::arguments::Cli;
use clap::Parser;



fn main() {
    let cli = Cli::parse();

    if let Err(e) = addressbook_1::run(cli) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
