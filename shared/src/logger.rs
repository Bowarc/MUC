fn colorise(message: String, level: log::Level) -> colored::ColoredString {
    use colored::Colorize as _;
    match level {
        log::Level::Trace => message.normal(),
        log::Level::Debug => message.cyan(),
        log::Level::Info => message.green(),
        log::Level::Warn => message.yellow(),
        log::Level::Error => message.red(),
        // _ => message.normal(),
    }
}

pub fn init(global_level: log::LevelFilter, log_file_opt: Option<&str>) {
    let mut builder = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "╭[{time} {level} {file_path}:{line_nbr}]\n╰❯{message}",
                time = chrono::Local::now().format("%H:%M:%S%.3f"),
                level = colorise(record.level().to_string(), record.level()),
                file_path = record.file().unwrap_or("Unknown file"),
                line_nbr = record
                    .line()
                    .map(|l| l.to_string())
                    .unwrap_or("?".to_string()),
                message = colorise(message.to_string(), record.level())
            ))
        })
        .level(global_level)
        .chain(std::io::stdout());
    if let Some(log_file) = log_file_opt {
        builder = builder.chain(fern::log_file(log_file).unwrap());
    }
    builder = builder.level_for("eframe", log::LevelFilter::Off);
    builder = builder.level_for("egui_glow", log::LevelFilter::Off);
    builder = builder.level_for("egui-winit-0.22.0", log::LevelFilter::Off);

    builder.apply().unwrap();

    log_panics::Config::new()
        .backtrace_mode(log_panics::BacktraceMode::Resolved)
        .install_panic_hook()
}
