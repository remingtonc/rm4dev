// Copyright (C) 2026 RM4 LLC
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::error::{AppError, AppResult};
use std::ffi::{OsStr, OsString};
use std::process::{Command, Output, Stdio};

pub(crate) fn run_for_output<I, S>(program: &str, args: I) -> AppResult<Output>
where
    I: IntoIterator<Item = S> + Clone,
    S: AsRef<OsStr>,
{
    let args_vec = args
        .clone()
        .into_iter()
        .map(|arg| arg.as_ref().to_os_string())
        .collect::<Vec<_>>();

    Command::new(program)
        .args(&args_vec)
        .output()
        .map_err(|source| AppError::Io {
            context: format!("failed to execute `{program}`"),
            source,
        })
}

pub(crate) fn run_and_capture<I, S>(program: &str, args: I) -> AppResult<String>
where
    I: IntoIterator<Item = S> + Clone,
    S: AsRef<OsStr>,
{
    let output = run_for_output(program, args.clone())?;
    let args_vec = args
        .into_iter()
        .map(|arg| arg.as_ref().to_os_string())
        .collect::<Vec<_>>();

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(AppError::CommandFailed {
            program: program.to_string(),
            args: render_os_args(&args_vec),
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

pub(crate) fn run_interactive(program: &str, args: Vec<OsString>) -> AppResult<()> {
    let status = Command::new(program)
        .args(&args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|source| AppError::Io {
            context: format!("failed to execute `{program}`"),
            source,
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(AppError::CommandFailed {
            program: program.to_string(),
            args: render_os_args(&args),
            status,
            stderr: String::new(),
        })
    }
}

pub(crate) fn render_os_args(args: &[OsString]) -> Vec<String> {
    args.iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect()
}
