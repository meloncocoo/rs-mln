use std::str::FromStr;

use log::info;
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
use serde::{de, Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum LogLevel {
    /// Only shows errors and warnings: `"critical"`.
    Critical,
    /// Shows everything except debug and trace information: `"normal"`.
    Normal,
    /// Shows everything: `"debug"`.
    Debug,
    /// Shows nothing: "`"off"`".
    Off,
}

impl From<LogLevel> for log::LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Critical => log::LevelFilter::Warn,
            LogLevel::Normal => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Off => log::LevelFilter::Off,
        }
    }
}

impl LogLevel {
    fn as_str(&self) -> &str {
        match self {
            LogLevel::Critical => "critical",
            LogLevel::Normal => "normal",
            LogLevel::Debug => "debug",
            LogLevel::Off => "off",
        }
    }
}

impl FromStr for LogLevel {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let level = match &*s.to_ascii_lowercase() {
            "critical" => LogLevel::Critical,
            "normal" => LogLevel::Normal,
            "debug" => LogLevel::Debug,
            "off" => LogLevel::Off,
            _ => return Err("a log level (off, debug, normal, critical)"),
        };

        Ok(level)
    }
}

impl Serialize for LogLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for LogLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        LogLevel::from_str(&string).map_err(|_| {
            de::Error::invalid_value(
                de::Unexpected::Str(&string),
                &figment::error::OneOf(&["critical", "normal", "debug", "off"]),
            )
        })
    }
}

// pub(crate) fn init_default() {
//     crate::log::init(&crate::Config::debug_default())
// }

pub fn init(log_file: &str, log_level: LogLevel) {
    let pattern = "🔧 [{l}] {d(%Y-%m-%d %H:%M:%S)} {file}:{line} - {m}{n}";
    let level: log::LevelFilter = log_level.into();

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .target(Target::Stdout)
        .build();

    // Logging to log file.
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build(log_file.to_string())
        .unwrap();

    // Log Trace level output to file where trace is the default level
    // and the programmatically specified level to stdout.
    let config = log4rs::config::Config::builder()
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(level)))
                .build("logfile", Box::new(logfile)),
        )
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(level)))
                .build("stdout", Box::new(stdout)),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stdout")
                .build(level),
        )
        .unwrap();

    // Use this to change log levels at runtime.
    // This means you can change the default log level to trace
    // if you are trying to debug an issue and need more logs on then turn it off
    // once you are done.
    let _handle = log4rs::init_config(config).unwrap();
    info!("log initialized.");
}
