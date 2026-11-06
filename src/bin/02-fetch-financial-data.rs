use divs_rs::sources::finmind;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    finmind::fetch_financial_data()?;
    Ok(())
}
