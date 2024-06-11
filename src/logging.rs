use spdlog::{prelude::*, sink::FileSink, sink::Sink};
use std::{env, path::PathBuf, sync::Arc};

/// Setup up spdlogger for console and file logging
pub fn logger_setup() {
    // Logging lines pulled from spdlog repo example
    const LOG_FILE: &str = "logs/transcript.log";

    // Set up file path
    let path: PathBuf = env::current_exe().unwrap().parent().unwrap().join(LOG_FILE);

    // logger for writing to file
    let file_sink: Arc<FileSink> = Arc::new(
        FileSink::builder()
            .path(&path)
            .truncate(true)
            .build()
            .expect("Failed to make file sink"),
    );

    // logger for console
    let mut sinks: Vec<Arc<dyn Sink>> = spdlog::default_logger().sinks().to_owned();
    // Combine file logger and console logger
    sinks.push(file_sink);

    let mut builder: LoggerBuilder = Logger::builder();
    let builder: &mut LoggerBuilder = builder.sinks(sinks).level_filter(LevelFilter::All);

    // Make a logger object
    let logger: Arc<Logger> = Arc::new(
        builder
            .name("logger")
            .build()
            .expect("Failed to build logger"),
    );
    // Flush when log "debug" and more severe logs.
    logger.set_flush_level_filter(LevelFilter::MoreSevereEqual(Level::Debug));

    // Make default logger to the system
    spdlog::set_default_logger(logger);
    // Logger
    warn!("LOGGING TO PATH: {:?}", path);
}
