use simplelog::{Config, LevelFilter, WriteLogger};

pub fn configure_logger() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let data_dir = dirs_next::data_dir().unwrap();
    let ce_log = data_dir.join(".ce_app.log");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(ce_log)?;

    WriteLogger::init(LevelFilter::Debug, Config::default(), file)?;
    Ok(())
}
