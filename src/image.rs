use crate::cli::ImageCommandArgs;
use crate::error::{AppError, AppResult};
use crate::process::{render_os_args, run_for_output, run_interactive};
use include_dir::{Dir, include_dir};
use std::collections::hash_map::DefaultHasher;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

pub(crate) const DEFAULT_IMAGE: &str = "localhost/rm4dev-agent:nix-fedora";
static NIX_FEDORA_CONTEXT: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/rm4dev-agent/nix-fedora");
const IMAGE_ENV: &str = "RM4DEV_IMAGE";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeImage {
    pub(crate) image: String,
    pub(crate) auto_ensure: bool,
}

pub(crate) fn runtime_image() -> RuntimeImage {
    runtime_image_from_env(env::var(IMAGE_ENV).ok())
}

pub(crate) fn runtime_image_from_env(image: Option<String>) -> RuntimeImage {
    match image {
        Some(image) => RuntimeImage {
            image,
            auto_ensure: false,
        },
        None => RuntimeImage {
            image: DEFAULT_IMAGE.to_string(),
            auto_ensure: true,
        },
    }
}

pub(crate) fn resolve_image_ref(image: Option<&str>) -> String {
    image.unwrap_or(DEFAULT_IMAGE).to_string()
}

pub(crate) fn ensure_runtime_image(runtime_image: &RuntimeImage) -> AppResult<()> {
    if runtime_image.auto_ensure {
        ensure_image_present(&runtime_image.image)?;
    }
    Ok(())
}

pub(crate) fn image_build(args: ImageCommandArgs) -> AppResult<()> {
    let image = resolve_image_ref(args.image.as_deref());
    build_embedded_image(&image)?;
    println!("built image {image}");
    Ok(())
}

pub(crate) fn image_ensure(args: ImageCommandArgs) -> AppResult<()> {
    let image = resolve_image_ref(args.image.as_deref());
    if image_exists(&image)? {
        println!("image already present: {image}");
        return Ok(());
    }

    build_embedded_image(&image)?;
    println!("built image {image}");
    Ok(())
}

fn ensure_image_present(image: &str) -> AppResult<()> {
    if image_exists(image)? {
        return Ok(());
    }

    build_embedded_image(image)
}

fn image_exists(image: &str) -> AppResult<bool> {
    let args = ["image", "exists", image];
    let output = run_for_output("podman", args)?;

    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(AppError::CommandFailed {
            program: "podman".to_string(),
            args: render_os_args(&args.map(OsString::from)),
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }),
    }
}

fn build_embedded_image(image: &str) -> AppResult<()> {
    let build_context = ensure_embedded_build_context()?;
    run_interactive(
        "podman",
        vec![
            "build".into(),
            "--tag".into(),
            image.into(),
            build_context.into_os_string(),
        ],
    )
}

fn ensure_embedded_build_context() -> AppResult<PathBuf> {
    let context_dir = image_cache_dir().join(embedded_context_hash());
    materialize_embedded_dir(&NIX_FEDORA_CONTEXT, &context_dir)?;
    Ok(context_dir)
}

fn image_cache_dir() -> PathBuf {
    if let Ok(path) = env::var("XDG_CACHE_HOME") {
        return PathBuf::from(path).join("rm4dev/images/nix-fedora");
    }

    if let Ok(path) = env::var("HOME") {
        return PathBuf::from(path).join(".cache/rm4dev/images/nix-fedora");
    }

    env::temp_dir().join("rm4dev/images/nix-fedora")
}

fn embedded_context_hash() -> String {
    let mut hasher = DefaultHasher::new();
    hash_embedded_dir(&NIX_FEDORA_CONTEXT, &mut hasher);
    format!("{:016x}", hasher.finish())
}

fn hash_embedded_dir(dir: &Dir<'_>, hasher: &mut DefaultHasher) {
    let mut dirs = dir.dirs().collect::<Vec<_>>();
    dirs.sort_by_key(|entry| entry.path());
    for child in dirs {
        child.path().to_string_lossy().hash(hasher);
        hash_embedded_dir(child, hasher);
    }

    let mut files = dir.files().collect::<Vec<_>>();
    files.sort_by_key(|entry| entry.path());
    for file in files {
        file.path().to_string_lossy().hash(hasher);
        file.contents().hash(hasher);
    }
}

fn materialize_embedded_dir(dir: &Dir<'_>, destination: &Path) -> AppResult<()> {
    fs::create_dir_all(destination).map_err(|source| AppError::Io {
        context: format!(
            "failed to create image build context `{}`",
            destination.display()
        ),
        source,
    })?;

    for child in dir.dirs() {
        fs::create_dir_all(destination.join(child.path())).map_err(|source| AppError::Io {
            context: format!(
                "failed to create image build context `{}`",
                destination.join(child.path()).display()
            ),
            source,
        })?;
        materialize_embedded_dir(child, destination)?;
    }

    for file in dir.files() {
        let path = destination.join(file.path());
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| AppError::Io {
                context: format!("failed to create directory `{}`", parent.display()),
                source,
            })?;
        }

        fs::write(&path, file.contents()).map_err(|source| AppError::Io {
            context: format!("failed to write embedded file `{}`", path.display()),
            source,
        })?;
    }

    Ok(())
}
