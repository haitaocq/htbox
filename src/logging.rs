use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("htbox logging initialized");
    Ok(())
}

pub fn init_with_file(log_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;

    let file = File::create(log_file)?;
    let (non_blocking, _guard) = tracing_appender::non_blocking(file);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .init();

    tracing::info!("htbox logging initialized with file: {}", log_file);
    Ok(())
}
