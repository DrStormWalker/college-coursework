extern crate pest;

#[macro_use]
extern crate pest_derive;

use std::collections::HashMap;
use std::fmt::Display;
use std::num::ParseIntError;

use chrono::{DateTime, FixedOffset};
use pest::Parser;

#[derive(Parser)]
#[grammar = "log.pest"]
struct LogParser;

use pest::{error::Error, iterators::Pairs};

#[derive(thiserror::Error, Debug)]
pub enum LogParseError {
    #[error("Failed to parse log: {0}")]
    SyntaxError(#[from] Error<Rule>),
    #[error("The field: `{0}` is required but not specified")]
    MissingFieldError(&'static str),
    #[error("The field: `{missing}` is required when the `{given}` field is specified")]
    MissingDebugFieldError {
        given: &'static str,
        missing: &'static str,
    },
    #[error("Failed to parse timestamp: {0}")]
    TimestampParseError(#[from] chrono::format::ParseError),
    #[error("Unknown option: `{option}` for field: `{field}`")]
    UnknownOptionError { option: String, field: &'static str },
    #[error("Failed to parse int for field: {field}: {source}")]
    IntegerParseError {
        field: &'static str,
        #[source]
        source: ParseIntError,
    },
}

#[derive(Debug)]
pub struct DebugInfo {
    file: String,
    line: u32,
}

#[derive(Debug)]
pub struct Log {
    time: DateTime<FixedOffset>,
    target: String,
    debug_info: Option<DebugInfo>,
    level: log::Level,
    msg: String,
    other: HashMap<String, String>,
}
impl Log {
    pub fn from_str(s: &str) -> Result<Self, LogParseError> {
        let map = parse_log_to_map(s)?;

        println!("map: {:?}", map);

        Self::from_map(map)
    }

    pub fn from_map(mut map: HashMap<String, String>) -> Result<Self, LogParseError> {
        #[inline(always)]
        fn get(
            map: &mut HashMap<String, String>,
            field: &'static str,
        ) -> Result<String, LogParseError> {
            map.remove(field)
                .ok_or(LogParseError::MissingFieldError(field))
        }

        Ok(Self {
            time: chrono::DateTime::<FixedOffset>::parse_from_rfc3339(
                get(&mut map, "time")?.as_str(),
            )?,
            target: get(&mut map, "target")?,
            debug_info: {
                match (get(&mut map, "file"), get(&mut map, "line")) {
                    (Ok(file), Ok(line)) => Some(DebugInfo {
                        file,
                        line: line
                            .parse()
                            .map_err(|err| LogParseError::IntegerParseError {
                                field: "line",
                                source: err,
                            })?,
                    }),
                    (Err(_), Ok(_)) => {
                        return Err(LogParseError::MissingDebugFieldError {
                            given: "line",
                            missing: "file",
                        })
                    }
                    (Ok(_), Err(_)) => {
                        return Err(LogParseError::MissingDebugFieldError {
                            given: "file",
                            missing: "line",
                        })
                    }
                    (Err(_), Err(_)) => None,
                }
            },
            level: {
                let level = get(&mut map, "level")?;
                match level.to_lowercase().as_str() {
                    "error" => log::Level::Error,
                    "warn" => log::Level::Warn,
                    "info" => log::Level::Info,
                    "debug" => log::Level::Debug,
                    "trace" => log::Level::Trace,
                    _ => {
                        return Err(LogParseError::UnknownOptionError {
                            option: level,
                            field: "level",
                        })
                    }
                }
            },
            msg: get(&mut map, "msg")?,
            other: map,
        })
    }

    pub fn time(&self) -> DateTime<FixedOffset> {
        self.time
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn file(&self) -> Option<&str> {
        Some(&self.debug_info.as_ref()?.file)
    }

    pub fn line(&self) -> Option<u32> {
        Some(self.debug_info.as_ref()?.line)
    }

    pub fn level(&self) -> log::Level {
        self.level
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }

    pub fn other(&self) -> &HashMap<String, String> {
        &self.other
    }

    pub fn is_debug_log(&self) -> bool {
        self.debug_info.is_some()
    }

    pub fn has_others(&self) -> bool {
        self.other.len() > 0
    }
}
impl Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "[{time}] [{target}] {debug_info}[{level}] {msg}{other}",
            time = self.time.format("%Y-%m-%d %H-%M-%S%.9f"),
            target = self.target,
            debug_info = self
                .debug_info
                .as_ref()
                .map_or("".to_string(), |v| format!("{}:{} ", v.file, v.line)),
            level = self.level,
            msg = self.msg,
            other = if self.has_others() {
                format!(
                    " | {}",
                    self.other
                        .iter()
                        .map(|(k, v)| format!("{}={:?}", k, v))
                        .fold(format!(""), |acc, v| format!("{} {}", acc, v)),
                )
            } else {
                format!("")
            }
        )
    }
}

pub fn parse_log_file(file: &str) -> Result<Vec<Log>, LogParseError> {
    parse_log_file_to_map(file)?
        .into_iter()
        .map(|map| Log::from_map(map))
        .collect()
}

fn parse_parameter(mut pairs: Pairs<Rule>) -> (String, String) {
    (
        pairs.next().unwrap().as_str().to_string(),
        pairs.next().unwrap().as_str().to_string(),
    )
}

fn parse_line(pairs: Pairs<Rule>) -> HashMap<String, String> {
    pairs
        .map(|pair| match pair.as_rule() {
            Rule::Parameter => parse_parameter(pair.into_inner()),
            _ => unreachable!(),
        })
        .collect()
}

fn parse_logs(pairs: Pairs<Rule>) -> Vec<HashMap<String, String>> {
    pairs
        .map(|pair| match pair.as_rule() {
            Rule::Line => parse_line(pair.into_inner()),
            _ => unreachable!(),
        })
        .collect()
}

fn parse_log_to_map(log: &str) -> Result<HashMap<String, String>, Error<Rule>> {
    let log = LogParser::parse(Rule::Line, log)?.next().unwrap();

    Ok(parse_line(log.into_inner()))
}

fn parse_log_file_to_map(file: &str) -> Result<Vec<HashMap<String, String>>, Error<Rule>> {
    let logs = LogParser::parse(Rule::File, file)?.next().unwrap();

    Ok(parse_logs(logs.into_inner()))
}
