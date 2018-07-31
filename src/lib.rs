//! os-release parser
//!
//! # Usage example
//!
//! ```
//! use rs_release::parse_os_release;
//!
//! let os_release_path = "/etc/os-release";
//! if let Ok(os_release) = parse_os_release(os_release_path) {
//!     println!("Parsed os-release:");
//!     for (k, v) in os_release {
//!         println!("{}={}", k, v);
//!     }
//! } else {
//!     println!("Cannot parse {}", os_release_path);
//! }
//! ```
#![deny(missing_docs)]

use std::collections::HashMap;
use std::convert::From;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::path::Path;
use std::borrow::Cow;

const PATHS: [&'static str; 2] = ["/etc/os-release", "/usr/lib/os-release"];
const QUOTES: [&'static str; 2] = ["\"", "'"];

const COMMON_KEYS: [&'static str; 16] = ["ANSI_COLOR",
                                         "BUG_REPORT_URL",
                                         "BUILD_ID",
                                         "CPE_NAME",
                                         "HOME_URL",
                                         "ID",
                                         "ID_LIKE",
                                         "NAME",
                                         "PRETTY_NAME",
                                         "PRIVACY_POLICY_URL",
                                         "SUPPORT_URL",
                                         "VARIANT",
                                         "VARIANT_ID",
                                         "VERSION",
                                         "VERSION_CODENAME",
                                         "VERSION_ID"];

/// Represents possible errors when parsing os-release file/string
#[derive(Debug)]
pub enum OsReleaseError {
    /// Input-Output error (failed to read file)
    Io(std::io::Error),
    /// Failed to find os-release file in standard paths
    NoFile,
    /// File is malformed
    ParseError,
}

impl PartialEq for OsReleaseError {
    fn eq(&self, other: &OsReleaseError) -> bool {
        match (self, other) {
            (&OsReleaseError::Io(_), &OsReleaseError::Io(_)) |
            (&OsReleaseError::NoFile, &OsReleaseError::NoFile) |
            (&OsReleaseError::ParseError, &OsReleaseError::ParseError) => true,
            _ => false,
        }
    }
}

impl fmt::Display for OsReleaseError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OsReleaseError::Io(ref inner) => inner.fmt(fmt),
            OsReleaseError::NoFile => write!(fmt, "{}", self.description()),
            OsReleaseError::ParseError => write!(fmt, "{}", self.description()),
        }
    }
}

impl Error for OsReleaseError {
    fn description(&self) -> &str {
        match *self {
            OsReleaseError::Io(ref err) => err.description(),
            OsReleaseError::NoFile => "Failed to find os-release file",
            OsReleaseError::ParseError => "File is malformed",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            OsReleaseError::Io(ref err) => Some(err),
            OsReleaseError::NoFile | OsReleaseError::ParseError => None,
        }
    }
}

impl From<std::io::Error> for OsReleaseError {
    fn from(err: std::io::Error) -> OsReleaseError {
        OsReleaseError::Io(err)
    }
}

/// A specialized `Result` type for os-release parsing operations.
pub type Result<T> = std::result::Result<T, OsReleaseError>;

fn trim_quotes(s: &str) -> &str {
    // TODO: is it malformed if we have only one quote?
    if QUOTES.iter().any(|q| s.starts_with(q) && s.ends_with(q)) {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

fn extract_variable_and_value(s: &str) -> Result<(Cow<'static, str>, String)> {
    if let Some(equal) = s.chars().position(|c| c == '=') {
        let var = &s[..equal];
        let var = var.trim();
        let val = &s[equal + 1..];
        let val = trim_quotes(val.trim()).to_string();

        if let Some(key) = COMMON_KEYS.iter().find(|&k| *k == var) {
            Ok((Cow::Borrowed(key), val))
        } else {
            Ok((Cow::Owned(var.to_string()), val))
        }
    } else {
        Err(OsReleaseError::ParseError)
    }
}

/// Parses key-value pairs from `path`
pub fn parse_os_release<P: AsRef<Path>>(path: P) -> Result<HashMap<Cow<'static, str>, String>> {
    let mut os_release = HashMap::new();
    let file = try!(File::open(path));
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = try!(line);
        let line = line.trim();

        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let var_val = try!(extract_variable_and_value(line));
        os_release.insert(var_val.0, var_val.1);
    }
    Ok(os_release)
}

/// Parses key-value pairs from `data` string
pub fn parse_os_release_str(data: &str) -> Result<HashMap<Cow<'static, str>, String>> {
    let mut os_release = HashMap::new();
    for line in data.split('\n') {
        let line = line.trim();

        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let var_val = try!(extract_variable_and_value(line));
        os_release.insert(var_val.0, var_val.1);
    }
    Ok(os_release)
}

/// Tries to find and parse os-release in common paths. Stops on success.
pub fn get_os_release() -> Result<HashMap<Cow<'static, str>, String>> {
    for file in &PATHS {
        if let Ok(os_release) = parse_os_release(file) {
            return Ok(os_release);
        }
    }
    Err(OsReleaseError::NoFile)
}
