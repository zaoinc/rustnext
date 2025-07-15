use env_logger::Env;
// Removed unused import: use log::LevelFilter;

pub fn init_logging() {
    let env = Env::default()
        .filter_or("RUST_LOG", "info")
        .write_style_or("RUST_LOG_STYLE", "always");

    env_logger::Builder::from_env(env)
        .format_timestamp_millis()
        .init();

    log::info!("Logging initialized.");
}