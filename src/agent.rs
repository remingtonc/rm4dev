// Copyright (C) 2026 RM4 LLC
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::cli::{ContainerTarget, CreateContainerArgs};
use crate::error::{AppError, AppResult};
use crate::image::{ensure_runtime_image, runtime_image};
use crate::mounts::MountSpec;
use crate::naming::{generate_container_name, is_agent_container_name};
use crate::process::{run_and_capture, run_interactive};
use std::env;
use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;

const HOST_AUTH_PATH: &str = ".cache/rm4dev/opencode-auth.json";
const CONTAINER_AUTH_PATH: &str = "/root/.local/share/opencode/auth.json";
const ENTER_SHELL_ENV: &str = "RM4DEV_ENTER_SHELL";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StartPlan {
    Create(CreateContainerArgs),
    Resume { name: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ListedContainer {
    name: String,
    image: String,
    status: String,
}

pub(crate) fn precheck() -> AppResult<()> {
    run_and_capture("podman", ["--version"])?;
    run_and_capture("podman", ["info"])?;
    println!("podman is installed and runnable");
    Ok(())
}

pub(crate) fn list_running_containers() -> AppResult<()> {
    let containers = load_agent_containers()?;
    print_container_table(&containers);
    Ok(())
}

pub(crate) fn create_container(args: CreateContainerArgs) -> AppResult<()> {
    let runtime_image = runtime_image();
    ensure_runtime_image(&runtime_image)?;
    let name = args.name.unwrap_or_else(generate_container_name);
    ensure_container_does_not_exist(&name)?;
    let podman_args = build_podman_run_args(
        &name,
        args.no_shared_auth,
        &args.mounts,
        &runtime_image.image,
    )?;
    run_interactive("podman", podman_args)
}

pub(crate) fn start_container(args: CreateContainerArgs) -> AppResult<()> {
    let existing = list_agent_container_names()?;
    match plan_start(existing, args)? {
        StartPlan::Create(args) => create_container(args),
        StartPlan::Resume { name } => resume_container(&name),
    }
}

pub(crate) fn stop_container(target: ContainerTarget) -> AppResult<()> {
    let name = resolve_existing_container(target)?;
    run_interactive("podman", vec!["stop".into(), name.into()])
}

pub(crate) fn remove_container(target: ContainerTarget) -> AppResult<()> {
    let name = resolve_existing_container(target)?;
    run_interactive("podman", vec!["rm".into(), name.into()])
}

pub(crate) fn attach_container(target: ContainerTarget) -> AppResult<()> {
    let name = resolve_existing_container(target)?;
    run_interactive("podman", vec!["attach".into(), name.into()])
}

pub(crate) fn enter_container(target: ContainerTarget) -> AppResult<()> {
    let name = resolve_existing_container(target)?;
    let state = container_state(&name)?;
    if state != "running" {
        return Err(AppError::Message(format!(
            "container `{name}` is `{state}`; start it before using `enter`"
        )));
    }

    run_interactive(
        "podman",
        vec![
            "exec".into(),
            "--interactive".into(),
            "--tty".into(),
            name.into(),
            enter_shell().into(),
            "-l".into(),
        ],
    )
}

pub(crate) fn enter_shell_from_env(shell: Option<String>) -> String {
    shell
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "/bin/bash".to_string())
}

fn enter_shell() -> String {
    enter_shell_from_env(env::var(ENTER_SHELL_ENV).ok())
}

fn load_agent_containers() -> AppResult<Vec<ListedContainer>> {
    let output = run_and_capture(
        "podman",
        [
            "ps",
            "--all",
            "--format",
            "{{.Names}}\t{{.Image}}\t{{.Status}}",
        ],
    )?;
    let mut containers = output
        .lines()
        .filter_map(parse_listed_container_line)
        .filter(|container| is_agent_container_name(&container.name))
        .collect::<Vec<_>>();

    containers.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(containers)
}

fn print_container_table(containers: &[ListedContainer]) {
    if containers.is_empty() {
        return;
    }

    let name_width = containers
        .iter()
        .map(|container| container.name.len())
        .max()
        .unwrap_or(4)
        .max(4);
    let image_width = containers
        .iter()
        .map(|container| container.image.len())
        .max()
        .unwrap_or(5)
        .max(5);

    println!(
        "{:<name_width$}  {:<image_width$}  STATUS",
        "NAME",
        "IMAGE",
        name_width = name_width,
        image_width = image_width
    );
    for container in containers {
        println!(
            "{:<name_width$}  {:<image_width$}  {}",
            container.name,
            container.image,
            container.status,
            name_width = name_width,
            image_width = image_width
        );
    }
}

fn parse_listed_container_line(line: &str) -> Option<ListedContainer> {
    if line.trim().is_empty() {
        return None;
    }

    let mut fields = line.splitn(3, '\t');
    let name = fields.next()?.trim().to_string();
    let image = fields.next()?.trim().to_string();
    let status = fields.next()?.trim().to_string();

    Some(ListedContainer {
        name,
        image,
        status,
    })
}

fn resolve_existing_container(target: ContainerTarget) -> AppResult<String> {
    let mut existing = list_agent_container_names()?;
    existing.sort();

    if let Some(name) = target.name {
        if existing.iter().any(|existing_name| existing_name == &name) {
            return Ok(name);
        }
        return Err(AppError::Message(format!(
            "container `{name}` was not found"
        )));
    }

    match existing.as_slice() {
        [] => Err(AppError::Message(
            "no rm4dev agent containers were found".to_string(),
        )),
        [name] => Ok(name.clone()),
        _ => Err(AppError::Message(
            "multiple rm4dev agent containers exist; specify a name".to_string(),
        )),
    }
}

fn list_agent_container_names() -> AppResult<Vec<String>> {
    let output = run_and_capture("podman", ["ps", "-a", "--format", "{{.Names}}"])?;
    let mut names = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| is_agent_container_name(line))
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    names.sort();
    Ok(names)
}

fn ensure_container_does_not_exist(name: &str) -> AppResult<()> {
    let existing = list_agent_container_names()?;
    if existing.iter().any(|existing_name| existing_name == name) {
        return Err(AppError::Message(format!(
            "container `{name}` already exists"
        )));
    }
    Ok(())
}

pub(crate) fn plan_start(existing: Vec<String>, args: CreateContainerArgs) -> AppResult<StartPlan> {
    if let Some(name) = args.name.clone() {
        if existing.iter().any(|existing_name| existing_name == &name) {
            if args.no_shared_auth || !args.mounts.is_empty() {
                return Err(AppError::Message(format!(
                    "container `{name}` already exists; create-only options only apply when creating a new container"
                )));
            }
            return Ok(StartPlan::Resume { name });
        }
        return Ok(StartPlan::Create(args));
    }

    if args.no_shared_auth || !args.mounts.is_empty() {
        return Ok(StartPlan::Create(args));
    }

    match existing.as_slice() {
        [] => Ok(StartPlan::Create(args)),
        [name] => Ok(StartPlan::Resume { name: name.clone() }),
        _ => Err(AppError::Message(
            "multiple rm4dev agent containers exist; specify a name".to_string(),
        )),
    }
}

fn resume_container(name: &str) -> AppResult<()> {
    match container_state(name)?.as_str() {
        "running" => run_interactive("podman", vec!["attach".into(), name.into()]),
        _ => run_interactive(
            "podman",
            vec![
                "start".into(),
                "--attach".into(),
                "--interactive".into(),
                name.into(),
            ],
        ),
    }
}

fn container_state(name: &str) -> AppResult<String> {
    let output = run_and_capture("podman", ["inspect", "--format", "{{.State.Status}}", name])?;
    Ok(output.trim().to_string())
}

pub(crate) fn build_podman_run_args(
    name: &str,
    no_shared_auth: bool,
    mounts: &[MountSpec],
    image: &str,
) -> AppResult<Vec<OsString>> {
    let cpus = cpu_quota();

    let mut args = vec![
        OsString::from("run"),
        OsString::from("--interactive"),
        OsString::from("--tty"),
        OsString::from("--name"),
        OsString::from(name),
        OsString::from("--privileged"),
        OsString::from("--cpus"),
        OsString::from(cpus.to_string()),
        OsString::from("--mount"),
        OsString::from("type=tmpfs,target=/tmp"),
        OsString::from("--mount"),
        OsString::from("type=tmpfs,target=/run"),
    ];

    if !no_shared_auth {
        let mount = cached_auth_mount()?;
        args.extend([
            OsString::from("--mount"),
            OsString::from(mount.podman_mount_arg()),
        ]);
    }

    for mount in mounts {
        args.extend([
            OsString::from("--mount"),
            OsString::from(mount.podman_mount_arg()),
        ]);
    }

    args.push(OsString::from(image));
    Ok(args)
}

fn cached_auth_mount() -> AppResult<MountSpec> {
    let host = cached_auth_host_path()?;
    Ok(MountSpec {
        host,
        container: CONTAINER_AUTH_PATH.to_string(),
    })
}

fn cached_auth_host_path() -> AppResult<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
        AppError::Message("HOME is not set; cannot resolve auth cache path".to_string())
    })?;
    let path = home.join(HOST_AUTH_PATH);
    let parent = path.parent().ok_or_else(|| {
        AppError::Message(format!(
            "failed to resolve parent directory for `{}`",
            path.display()
        ))
    })?;

    fs::create_dir_all(parent).map_err(|source| AppError::Io {
        context: format!(
            "failed to create auth cache directory `{}`",
            parent.display()
        ),
        source,
    })?;

    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&path)
        .map_err(|source| AppError::Io {
            context: format!("failed to initialize auth cache file `{}`", path.display()),
            source,
        })?;

    fs::canonicalize(&path).map_err(|source| AppError::Io {
        context: format!("failed to resolve auth cache file `{}`", path.display()),
        source,
    })
}

pub(crate) fn cpu_quota() -> usize {
    let total = std::thread::available_parallelism()
        .map(|parallelism| parallelism.get())
        .unwrap_or(1);
    let reserved = total / 4;
    total.saturating_sub(reserved).max(1)
}
