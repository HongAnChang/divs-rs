use divs_rs::sources::twse;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    twse::fetch_company_info()?;
    Ok(())
}
