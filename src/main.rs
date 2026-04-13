use htbox::cli;
use htbox::logging;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::init()?;
    cli::run()?;
    Ok(())
}
