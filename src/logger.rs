use log::SetLoggerError;
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config, Handle,
};

pub(crate) fn init_logger(logger_level: &str) -> Result<Handle, SetLoggerError> {
    // logger level manager
    let logger_level = match logger_level {
        "TRACE" => log::LevelFilter::Trace,
        "DEBUG" => log::LevelFilter::Debug,
        "INFO" => log::LevelFilter::Info,
        "WARN" => log::LevelFilter::Warn,
        "ERROR" => log::LevelFilter::Error,
        "QUIET" => log::LevelFilter::Off,
        _ => log::LevelFilter::Info,
    };

    let console = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{l}] {d} - {m}{n}")))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("console", Box::new(console)))
        .build(Root::builder().appender("console").build(logger_level))
        .unwrap();

    let logger = log4rs::init_config(config);

    if logger_level == log::LevelFilter::Trace || logger_level == log::LevelFilter::Debug {
        log::debug!(
            "You have activated the `debug` log mode, is not suitable for use in a production env!"
        );
    }

    logger
}
