// Copyright (C) 2026 RM4 LLC
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::error::{AppError, AppResult};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MountSpec {
    pub(crate) host: PathBuf,
    pub(crate) container: String,
}

impl MountSpec {
    pub(crate) fn podman_mount_arg(&self) -> String {
        format!(
            "type=bind,src={},target={}",
            self.host.display(),
            self.container
        )
    }
}

pub(crate) fn looks_like_mount_spec(candidate: &str) -> bool {
    candidate.rsplit_once(':').is_some()
}

pub(crate) fn parse_mount_spec(input: &str) -> AppResult<MountSpec> {
    let Some((host, container)) = input.rsplit_once(':') else {
        return Err(AppError::Usage(format!(
            "invalid mount `{input}`; expected host_path:container_path"
        )));
    };

    if host.is_empty() || container.is_empty() {
        return Err(AppError::Usage(format!(
            "invalid mount `{input}`; both host and container paths are required"
        )));
    }

    if !container.starts_with('/') {
        return Err(AppError::Usage(format!(
            "invalid mount `{input}`; container path must be absolute"
        )));
    }

    let canonical_host = fs::canonicalize(host).map_err(|source| AppError::Io {
        context: format!("failed to resolve mount source `{host}`"),
        source,
    })?;

    Ok(MountSpec {
        host: canonical_host,
        container: container.to_string(),
    })
}
