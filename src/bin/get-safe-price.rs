use divs_rs::safe_price;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    safe_price::calculate()?;
    Ok(())
}
