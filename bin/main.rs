const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> miette::Result<()> {
    println!("Cardboard v{}", VERSION);
    Ok(())
}
