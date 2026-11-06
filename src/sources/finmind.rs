mod cli;
mod fetcher;

use clap::Parser;
use cli::Args;

pub fn fetch_financial_data() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    fetcher::fetch_financial_data(args.dataset.as_str())?;
    Ok(())
}
