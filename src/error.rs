// Copyright (C) 2026 RM4 LLC
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fmt;
use std::process::ExitStatus;

#[derive(Debug)]
pub(crate) enum AppError {
    Cli {
        message: String,
        exit_code: i32,
    },
    Usage(String),
    Message(String),
    Io {
        context: String,
        source: std::io::Error,
    },
    CommandFailed {
        program: String,
        args: Vec<String>,
        status: ExitStatus,
        stderr: String,
    },
}

impl AppError {
    pub(crate) fn exit_code(&self) -> i32 {
        match self {
            Self::Cli { exit_code, .. } => *exit_code,
            Self::Usage(_) => 2,
            Self::Message(_) | Self::Io { .. } | Self::CommandFailed { .. } => 1,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cli { message, .. } => write!(f, "{message}"),
            Self::Usage(message) | Self::Message(message) => write!(f, "{message}"),
            Self::Io { context, source } => write!(f, "{context}: {source}"),
            Self::CommandFailed {
                program,
                args,
                status,
                stderr,
            } => {
                let rendered_args = if args.is_empty() {
                    String::new()
                } else {
                    format!(" {}", args.join(" "))
                };
                let rendered_status = status.code().map_or_else(
                    || "terminated by signal".to_string(),
                    |code| code.to_string(),
                );
                let stderr = stderr.trim();

                if stderr.is_empty() {
                    write!(
                        f,
                        "command failed: {program}{rendered_args} (exit {rendered_status})"
                    )
                } else {
                    write!(
                        f,
                        "command failed: {program}{rendered_args} (exit {rendered_status}): {stderr}"
                    )
                }
            }
        }
    }
}

impl std::error::Error for AppError {}

pub(crate) type AppResult<T> = Result<T, AppError>;
