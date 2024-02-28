// Copied from: https://github.com/rust-lang/crates.io/blob/main/crates_io_env_vars/src/lib.rs
// License: MIT/Apache-2.0
//
// Testing is not included because the original source ties with anyhow but
// we're using error-stack for error management.

use derive_more::Display;
use error_stack::{Context, Report, Result, ResultExt};
use std::str::FromStr;

#[derive(Debug, Display)]
#[display(fmt = "Could not read {_0:?} environment variable")]
pub struct ReadVarError(&'static str);

impl error_stack::Context for ReadVarError {}

/// Reads an environment variable for the current process.
///
/// Compared to [std::env::var] there are a couple of differences:
///
/// - [var] uses [dotenvy] which loads the `.env` file from the current or
///   parent directories before returning the value.
///
/// - [var] returns `Ok(None)` (instead of `Err`) if an environment variable
///   wasn't set.
#[track_caller]
pub fn var(key: &'static str) -> Result<Option<String>, ReadVarError> {
    match dotenvy::var(key) {
        Ok(content) => Ok(Some(content)),
        Err(dotenvy::Error::EnvVar(std::env::VarError::NotPresent)) => Ok(None),
        Err(error) => Err(error).change_context(ReadVarError(key)),
    }
}

/// Reads an environment variable for the current process, and fails if it was
/// not found.
///
/// Compared to [std::env::var] there are a couple of differences:
///
/// - [var] uses [dotenvy] which loads the `.env` file from the current or
///   parent directories before returning the value.
#[track_caller]
pub fn required_var(key: &'static str) -> Result<String, ReadVarError> {
    required(var(key), key)
}

/// Reads an environment variable for the current process, and parses it if
/// it is set.
///
/// Compared to [std::env::var] there are a couple of differences:
///
/// - [var] uses [dotenvy] which loads the `.env` file from the current or
///   parent directories before returning the value.
///
/// - [var] returns `Ok(None)` (instead of `Err`) if an environment variable
///   wasn't set.
#[track_caller]
pub fn var_parsed<R, E>(key: &'static str) -> Result<Option<R>, ReadVarError>
where
    R: FromStr,
    E: Context,
    R::Err: IntoReport<E> + Send + Sync + 'static,
{
    match var(key) {
        Ok(Some(content)) => Ok(Some(
            content
                .parse::<R>()
                .map_err(|e| e.into_report())
                .change_context(ReadVarError(key))
                .attach_printable("couldn't parse environment variable")?,
        )),
        Ok(None) => Ok(None),
        Err(error) => Err(error),
    }
}

/// Reads an environment variable for the current process, and parses it if
/// it is set or fails otherwise.
///
/// Compared to [std::env::var] there are a couple of differences:
///
/// - [var] uses [dotenvy] which loads the `.env` file from the current or
///   parent directories before returning the value.
#[track_caller]
pub fn required_var_parsed<R, E>(key: &'static str) -> Result<R, ReadVarError>
where
    R: FromStr,
    E: Context,
    R::Err: IntoReport<E> + Send + Sync + 'static,
{
    required(var_parsed(key), key)
}

fn required<T>(res: Result<Option<T>, ReadVarError>, key: &'static str) -> Result<T, ReadVarError> {
    match res {
        // TODO: Find a good word/sentence for this
        Ok(opt) => opt.ok_or_else(|| {
            Report::new(ReadVarError(key)).attach_printable("environment variable is missing")
        }),
        Err(error) => Err(error),
    }
}

/// Reads an environment variable and parses it as a comma-separated list, or
/// returns an empty list if the variable is not set.
#[track_caller]
pub fn list(key: &'static str) -> Result<Vec<String>, ReadVarError> {
    let values = match var(key)? {
        None => vec![],
        Some(s) if s.is_empty() => vec![],
        Some(s) => s.split(',').map(str::trim).map(String::from).collect(),
    };

    Ok(values)
}

/// Reads an environment variable and parses it as a comma-separated list, or
/// returns an empty list if the variable is not set. Each individual value is
/// parsed using [FromStr].
#[track_caller]
pub fn list_parsed<T, E, F, C>(key: &'static str, f: F) -> Result<Vec<T>, ReadVarError>
where
    F: Fn(&str) -> ::std::result::Result<T, E>,
    E: IntoReport<C>,
    C: Context,
{
    let values = match var(key)? {
        None => vec![],
        Some(s) if s.is_empty() => vec![],
        Some(s) => s
            .split(',')
            .map(str::trim)
            .map(|s| {
                f(s).map_err(|e| e.into_report())
                    .change_context(ReadVarError(key))
                    .attach_printable_lazy(|| format!("failed to parse value \"{s}\""))
            })
            .collect::<Result<_, _>>()?,
    };

    Ok(values)
}

pub trait IntoReport<T: Context> {
    fn into_report(self) -> Report<T>;
}

impl<T: Context> IntoReport<T> for T {
    fn into_report(self) -> Report<T> {
        Report::new(self)
    }
}

impl<T: Context> IntoReport<T> for Report<T> {
    fn into_report(self) -> Report<T> {
        self
    }
}
