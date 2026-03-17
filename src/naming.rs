use crate::error::{AppError, AppResult};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) const CONTAINER_PREFIX: &str = "rm4dev-agent-";

pub(crate) fn normalize_container_name(input: &str) -> AppResult<String> {
    if input.trim().is_empty() {
        return Err(AppError::Usage(
            "container name cannot be empty".to_string(),
        ));
    }

    let normalized = if input.starts_with(CONTAINER_PREFIX) {
        input.to_string()
    } else {
        format!("{CONTAINER_PREFIX}{input}")
    };

    if is_valid_container_name(&normalized) {
        Ok(normalized)
    } else {
        Err(AppError::Usage(format!(
            "invalid container name `{input}`; use letters, numbers, `.`, `_`, or `-`"
        )))
    }
}

pub(crate) fn is_valid_container_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(ch) if ch.is_ascii_alphanumeric() => (),
        _ => return false,
    }

    chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
}

pub(crate) fn is_agent_container_name(name: &str) -> bool {
    name.starts_with(CONTAINER_PREFIX)
}

pub(crate) fn generate_container_name() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{CONTAINER_PREFIX}{seconds}")
}
