use clap::Parser;
use std::path::PathBuf;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]

pub struct Args {
    
    // Append Session id into file
    #[arg(short, long, help = "config file path")]
    pub config: Option<PathBuf>,

}

pub fn arg_parser() -> Args {
    let args: Args = Args::parse();
    args
}
