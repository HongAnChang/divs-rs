use clap::Parser;
use divs_rs::{cli::Args, finmind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    finmind::fetch_financial_data(args.dataset.as_str())?;
    Ok(())
}
