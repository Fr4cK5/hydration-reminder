use std::{env, fs, time::Duration};

use eyre::{OptionExt, Result, eyre};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(not(debug_assertions))]
const DURATION: Duration = Duration::from_secs(20 * 60);

#[cfg(debug_assertions)]
const DURATION: Duration = Duration::from_secs(5);

#[cfg(not(debug_assertions))]
const FS_DURATION: &'static str = "20m";

#[cfg(debug_assertions)]
const FS_DURATION: &'static str = "5s";

// TODO: Implement json schema generation using clap
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct FSConfig {
    /// A low-resolution duration in simple string form.
    ///
    /// It's composed of any number and a suffix, repeated as many times as one likes.
    ///
    /// The suffix can be any of
    /// - s (seconds)
    /// - m (minutes)
    /// - h (hours)
    ///
    /// # Examples
    /// All of the following produce a valid output [`Duration`]
    /// - `30m`
    /// - `20m30s`
    /// - `10h10m10s`
    /// - `10s2h`
    /// - `20s20s80s`
    reminder_interval: String,
}

impl Default for FSConfig {
    fn default() -> Self {
        Self {
            reminder_interval: String::from(FS_DURATION),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub reminder_interval: Duration,
    pub time_parsing_failed: bool,
    pub is_default: bool,
}

impl Config {
    pub fn try_from_path(path: &str) -> Result<Self> {
        eprintln!("Reading from: {path} in dir {:?}", env::current_dir());

        fs::read_to_string(path)
            .map_err(|e| eyre::Report::new(e))
            .map(|content| serde_json::from_str::<FSConfig>(&content).unwrap_or_default())
            .map(|fs_config| Config::from(fs_config))
    }
}

impl From<FSConfig> for Config {
    fn from(value: FSConfig) -> Self {
        let reminder_interval = DurationParser::new(&value.reminder_interval).get();
        let time_parsing_failed = reminder_interval.is_err();
        let reminder_interval = reminder_interval.unwrap_or(DURATION);

        dbg!(&reminder_interval);
        Self {
            reminder_interval,
            time_parsing_failed,
            is_default: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            reminder_interval: DURATION,
            time_parsing_failed: false,
            is_default: true,
        }
    }
}

#[derive(Debug)]
struct DurationParser<'a> {
    value: &'a str,
    pos: usize,
}

impl<'a> DurationParser<'a> {
    pub fn new(value: &'a str) -> Self {
        Self {
            value: value.trim(),
            pos: 0,
        }
    }

    pub fn get(&mut self) -> Result<Duration> {
        if self.value.is_empty() {
            return Err(eyre!("Cannot parse from empty string."));
        }

        let mut sec_duration = 0u64;
        while self.pos < self.value.len() {
            eprintln!("Parsing: {}", &self.value[self.pos..]);

            let raw_time = self.parse_number()?;
            eprintln!("Parsing: {}", &self.value[self.pos..]);

            let time_mult = self.parse_suffix_into_multiplier()?;
            eprintln!("Parsing: {}", &self.value[self.pos..]);

            sec_duration += raw_time * time_mult;
        }

        Ok(Duration::from_secs(sec_duration))
    }

    fn parse_number(&mut self) -> Result<u64> {
        let buf = self.value[self.pos..]
            .chars()
            .take_while(|c| c.is_numeric())
            .collect::<String>();

        if buf.is_empty() {
            return Err(eyre!("No numeric value found"));
        }

        let x = buf.parse::<u64>()?;
        self.pos += buf.len();

        Ok(x)
    }

    fn parse_suffix_into_multiplier(&mut self) -> Result<u64> {
        let buf = self.value[self.pos..].chars().next().ok_or_eyre(
            "No duration specifier, use suffixes such as s, m, h in the interval: 20m30s, 25m",
        )?;

        self.pos += 1;

        match buf {
            's' => Ok(1),
            'm' => Ok(60),
            'h' => Ok(60 * 60),
            _ => Err(eyre!("Invalid duration suffix")),
        }
    }
}

#[cfg(test)]
mod parser_test {
    use std::time::Duration;

    use crate::config::DurationParser;

    #[test]
    fn single() {
        let time = DurationParser::new("10m").get().unwrap();
        assert_eq!(time, Duration::from_secs(600));
    }

    #[test]
    fn multi() {
        let time = DurationParser::new("10m30s").get().unwrap();
        assert_eq!(time, Duration::from_secs(630));
    }

    #[test]
    fn multi_all() {
        let time = DurationParser::new("1h10m30s").get().unwrap();
        assert_eq!(time, Duration::from_secs(3600 + 630));
    }

    #[test]
    fn fail_empty() {
        let time = DurationParser::new("").get();
        assert!(time.is_err());
    }

    #[test]
    fn fail_whitespace_only() {
        let time = DurationParser::new(" \t \r\n \n ").get();
        assert!(time.is_err());
    }

    #[test]
    fn fail_wrong_suffix() {
        let time = DurationParser::new("10x").get();
        assert!(time.is_err());
    }
}
