use log4rs::{Config, Handle};
use log4rs::config::{Appender, Logger, Root};
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log::{LevelFilter, SetLoggerError};
use log4rs::append::console::ConsoleAppender;

pub fn init_logger(path: String, quiet: bool) -> Result<Handle, SetLoggerError> {

    let root = &path;

    let record = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{l}] {d} - {m}{n}")))
        .build();

    let quiet_record = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{l}] {d} - {m}{n}")))
        .build(format!("{}/log/record.log",root))
        .unwrap();

    let eliminate = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{l}] {d} - {m}{n}")))
        .build(format!("{}/log/expired.log",root))
        .unwrap();

    let curd = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{l}] {d} - {m}{n}")))
        .build(format!("{}/log/curd.log",root))
        .unwrap();


    let server = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{l}] {d} - {m}{n}")))
        .build(format!("{}/log/server.log",root))
        .unwrap();

    let handle = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{l}] {d} - {m}{n}")))
        .build(format!("{}/log/handle.log",root))
        .unwrap();

    let builder = Config::builder()
        .appender(Appender::builder().build("record",Box::new(record)))
        .appender(Appender::builder().build("quiet",Box::new(quiet_record)))
        .appender(Appender::builder().build("eliminate",Box::new(eliminate)))
        .appender(Appender::builder().build("curd",Box::new(curd)))
        .appender(Appender::builder().build("server",Box::new(server)))
        .appender(Appender::builder().build("handle",Box::new(handle)))
        .logger(Logger::builder().appender("eliminate").build("dorea::structure",LevelFilter::Info))
        .logger(Logger::builder().appender("curd").build("dorea::database",LevelFilter::Info))
        .logger(Logger::builder().appender("server").build("dorea::server",LevelFilter::Info))
        .logger(Logger::builder().appender("handle").build("dorea::handle",LevelFilter::Info));

    let config: Config;
    if quiet {
        config = builder
            .build(Root::builder().appender("quiet").build(LevelFilter::Info))
            .unwrap();
    } else {
        config = builder
            .build(Root::builder().appender("record").build(LevelFilter::Info))
            .unwrap();
    }

    log4rs::init_config(config)
}