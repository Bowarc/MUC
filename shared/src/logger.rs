pub fn init(log_file_opt: Option<&str>) {
    let mut builder = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "╭[{time} {level} {file_path}:{line_nbr}]\n╰❯{message}",
                time = chrono::Local::now().format("%H:%M:%S%.3f"),
                level = record.level(),
                file_path = record.file().unwrap_or("Unknown file"),
                line_nbr = record
                    .line()
                    .map(|l| l.to_string())
                    .unwrap_or("?".to_string()),
                message = message
            ))
        })
        .level(log::LevelFilter::Error)
        .chain(std::io::stdout());
    if let Some(log_file) = log_file_opt {
        builder = builder.chain(fern::log_file(log_file).unwrap());
    }
    builder.apply().unwrap();

    log_panics::Config::new()
        .backtrace_mode(log_panics::BacktraceMode::Resolved)
        .install_panic_hook()
}
