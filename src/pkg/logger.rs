/*use std::fs;
use chrono::Local;
use fern::Dispatch;
use log::LevelFilter;

pub fn init_logger() {
    // Create logs directory if not exists
    fs::create_dir_all("logs").unwrap();

    let log_file = format!("logs/{}.log", Local::now().format("%Y-%m-%d"));

    Dispatch::new()
        // Console output (colored)
        .chain(
            fern::Dispatch::new()
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "[{}][{}][{}] {}",
                        Local::now().format("%Y-%m-%d %H:%M:%S"),
                        record.level(),
                        record.target(),
                        message
                    ))
                })
                .level(LevelFilter::Info)
                .chain(std::io::stdout()),
        )
        // File output (no colors)
        .chain(
            fern::Dispatch::new()
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "[{}][{}][{}] {}",
                        Local::now().format("%Y-%m-%d %H:%M:%S"),
                        record.level(),
                        record.target(),
                        message
                    ))
                })
                .level(LevelFilter::Debug)
                .chain(fern::log_file(log_file).unwrap()),
        )
        .apply()
        .unwrap();
}*/