use clap::Parser;
use color_eyre::Result;

#[derive(Parser, Debug)]
struct CLI {}

impl CLI {
    fn run(&self) -> Result<()> {
        println!("{self:#?}");

        Ok(())
    }
}

fn main() {
    color_eyre::install().unwrap();

    let cli = CLI::parse();
    if let Err(err) = cli.run() {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
