use log4rs::{Config, Handle};
use log4rs::config::{Appender, Logger, Root};
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log::{LevelFilter, SetLoggerError};

pub fn init_logger(root: &'static str) -> Result<Handle, SetLoggerError> {

    let record = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{l}] {d} - {m}{n}")))
        .build(format!("{}/log/dorea.log",root))
        .unwrap();

    let eliminate = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[URL] {d} - {m}{n}")))
        .build(format!("{}/log/eliminate.log",root))
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("record",Box::new(record)))
        .appender(Appender::builder().build("eliminate",Box::new(eliminate)))
        .logger(Logger::builder().appender("eliminate").build("dorea::structure",LevelFilter::Info))
        .build(Root::builder().appender("record").build(LevelFilter::Info))
        .unwrap();

    log4rs::init_config(config)
}