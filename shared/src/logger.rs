use {
    colored::Colorize,
    std::{
        cmp::Ordering,
        io::Write,
        sync::atomic::{AtomicUsize, Ordering as AtomicOrdering},
    },
};

static MAX_MODULE_WIDTH: AtomicUsize = AtomicUsize::new(6);

#[derive(Debug)]
pub struct LoggerConfig {
    global_level: log::LevelFilter,
    filters: Vec<(String, log::LevelFilter)>,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl LoggerConfig {
    pub fn new() -> Self {
        Self {
            global_level: log::LevelFilter::Trace,
            filters: vec![],
        }
    }
    pub fn set_level(mut self, level: log::LevelFilter) -> Self {
        self.global_level = level;
        self
    }
    pub fn add_filter(mut self, name: &str, level: log::LevelFilter) -> Self {
        self.filters.push((name.to_string(), level));
        self
    }
}

fn max_msg_width(msg: &str) -> usize {
    let max_width = MAX_MODULE_WIDTH.load(AtomicOrdering::Relaxed);
    if max_width < msg.len() {
        MAX_MODULE_WIDTH.store(msg.len(), AtomicOrdering::Relaxed);
        msg.len()
    } else {
        max_width
    }
}
fn set_color(message: String, level: log::Level) -> colored::ColoredString {
    match level {
        log::Level::Trace => message.normal(),
        log::Level::Debug => message.cyan(),
        log::Level::Info => message.green(),
        log::Level::Warn => message.yellow(),
        log::Level::Error => message.red(),
        // _ => message.normal(),
    }
}

pub fn init(config: LoggerConfig) {
    let mut builder = &mut env_logger::Builder::new();
    builder.format(|f, record| {
        let module_path = record.target().split("::").collect::<Vec<&str>>();
        let path = {
            let len = module_path.len();
            match len.cmp(&2) {
                Ordering::Greater => format!(
                    "{}::{}",
                    module_path[0],
                    module_path[len - 2..len].to_vec().join("::")
                ),
                Ordering::Equal => module_path[len - 2..len].to_vec().join("::"),
                Ordering::Less => module_path.join(""),
            }
        };

        let path_level = format!("{} - {}", path, record.level());
        // let path_level = format!("{} ", record.level());
        let max_width = max_msg_width(&path_level);
        let path_level_sized = format!(
            "{path_level: <width$}",
            path_level = path_level,
            width = max_width,
        );

        let final_formated_message = format!("{}| {}", path_level_sized, record.args());

        writeln!(f, "{}", set_color(final_formated_message, record.level()))
    });
    builder = builder.filter_level(config.global_level);

    for filter in config.filters.iter() {
        builder = builder.filter(Some(&filter.0), filter.1);
    }

    let default_panic_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        let payload = match panic_info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match panic_info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<dyn Any>",
            },
        };

        // let splitted = payload.split("value: ").collect::<Vec<&str>>();

        // let msg = if splitted.len() > 1 {
        //     splitted[1].replace('\\', "").replace('\"', "")
        // } else {
        //     payload.to_string()
        // };

        // Bowarc - Comment - Very unstable
        let formated_target = &panic_info
            .location()
            .unwrap()
            .file()
            .to_string()
            .replace("src", env!("CARGO_PKG_NAME"))
            .replace('\\', "::")
            .replace("mod.rs", "")
            .replace("main.rs", "")
            .replace(".rs", "");

        error!(
            target: formated_target,
            "{}\nLoc: {}",
            payload.replace(['\\', '\"'], ""),
            panic_info.location().unwrap()
        );

        // console_log(

        //         .split("::")
        //         .filter(|e| !e.is_empty())
        //         .collect::<Vec<&str>>(),
        //     format_args!(
        //         "{}\nLoc: {}",
        //         payload.replace('\\', "").replace('\"', ""),
        //         panic_info.location().unwrap()
        //     ),
        //     Level::Error,
        // )
        // .unwrap();
        // error!("panic occurred with message: {:?}", msg);

        default_panic_hook(panic_info)
    }));

    builder.init();
}

pub fn test() {
    trace!("This is Trace level"); // target: "custom_target",
    debug!("This is Debug level");
    info!("This is Info level");
    warn!("This is Warn level");
    error!("This is Error level");
}
