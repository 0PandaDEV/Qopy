use chrono;
use log::{ LevelFilter, SetLoggerError };
use std::fs::{ File, OpenOptions };
use std::io::Write;
use std::panic;

pub struct FileLogger {
    file: File,
}

impl log::Log for FileLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let mut file = self.file.try_clone().expect("Failed to clone file handle");

            // Format: timestamp [LEVEL] target: message (file:line)
            writeln!(
                file,
                "{} [{:<5}] {}: {} ({}:{})",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                record.args(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0)
            ).expect("Failed to write to log file");
        }
    }

    fn flush(&self) {
        self.file.sync_all().expect("Failed to flush log file");
    }
}

pub fn init_logger(app_data_dir: &std::path::Path) -> Result<(), SetLoggerError> {
    let logs_dir = app_data_dir.join("logs");
    std::fs::create_dir_all(&logs_dir).expect("Failed to create logs directory");

    // Use .log extension for standard log files
    let log_path = logs_dir.join("app.log");
    let file = OpenOptions::new()
        .create(true)
        .append(true) // Use append mode instead of write
        .open(&log_path)
        .expect("Failed to open log file");

    // Set up panic hook
    let panic_file = file.try_clone().expect("Failed to clone file handle");
    panic::set_hook(
        Box::new(move |panic_info| {
            let mut file = panic_file.try_clone().expect("Failed to clone file handle");

            let location = panic_info
                .location()
                .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
                .unwrap_or_else(|| "unknown location".to_string());

            let message = match panic_info.payload().downcast_ref::<&str>() {
                Some(s) => *s,
                None =>
                    match panic_info.payload().downcast_ref::<String>() {
                        Some(s) => s.as_str(),
                        None => "Unknown panic message",
                    }
            };

            let _ = writeln!(
                file,
                "{} [PANIC] rust_panic: {} ({})",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                message,
                location
            );
        })
    );

    let logger = Box::new(FileLogger { file });
    unsafe {
        log::set_logger_racy(Box::leak(logger))?;
    }
    log::set_max_level(LevelFilter::Debug);
    Ok(())
}
