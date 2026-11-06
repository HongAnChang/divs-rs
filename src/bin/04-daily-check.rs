use divs_rs::notify;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    notify::check()?;
    Ok(())
}
