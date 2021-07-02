use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::{Config, Handle};
use log4rs::config::{Appender, Logger, Root};
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log::{LevelFilter, SetLoggerError};
use log4rs::append::console::ConsoleAppender;

pub fn init_logger(path: String, quiet: bool) -> Result<Handle, SetLoggerError> {

    let root = path.clone() + "/log/";

    let size_limit = 30 * 1024; // 30KB as max log file size to roll
    let size_trigger = SizeTrigger::new(size_limit);

    let window_size = 20;
    let format = format!("{}/overdue/manager{}.log", root, "{}");
    let fixed_window_roller = FixedWindowRoller::builder()
        .build(&format,window_size).unwrap();

    let compound_policy = CompoundPolicy::new(
        Box::new(size_trigger),
        Box::new(fixed_window_roller)
    );

    let record = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{l}] {d} - {m}{n}")))
        .build();

    let quiet_record = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{l}] {d} - {m}{n}")))
        .build(format!("{}/record.log",root))
        .unwrap();

    let eliminate = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{l}] {d} - {m}{n}")))
        .build(format!("{}/expired.log",root))
        .unwrap();

    // let curd = FileAppender::builder()
    //     .encoder(Box::new(PatternEncoder::new("[{l}] {d} - {m}{n}")))
    //     .build(format!("{}/curd.log",root))
    //     .unwrap();

    let curd = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{l}] {d} - {m}{n}")))
        .build(format!("{}/curd.log",root),Box::new(compound_policy))
        .unwrap();

    let server = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{l}] {d} - {m}{n}")))
        .build(format!("{}/server.log",root))
        .unwrap();

    let handle = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{l}] {d} - {m}{n}")))
        .build(format!("{}/handle.log",root))
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