// Copyright (C) 2026 RM4 LLC
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::cli::{ContainerTarget, CreateContainerArgs};
use crate::error::{AppError, AppResult};
use crate::image::{ensure_runtime_image, runtime_image};
use crate::mounts::MountSpec;
use crate::naming::{CONTAINER_PREFIX, generate_container_name};
use crate::process::{run_and_capture, run_interactive};
use std::collections::BTreeSet;
use std::env;
use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

const HOST_AUTH_PATH: &str = ".cache/rm4dev/opencode-auth.json";
const CONTAINER_AUTH_PATH: &str = "/root/.local/share/opencode/auth.json";
const CONTAINER_WEB_PORT_ENV: &str = "RM4DEV_OPENCODE_WEB_PORT";
const CONTAINER_WEB_PORT_LABEL: &str = "org.rm4dev.opencode.web-port";
const CONTAINER_WEB_PASSWORD_ENV: &str = "OPENCODE_SERVER_PASSWORD";
const ENTER_SHELL_ENV: &str = "RM4DEV_ENTER_SHELL";
const WEB_PORT_START: u16 = 35080;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StartPlan {
    Create(CreateContainerArgs),
    Resume { name: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WebSettings {
    pub(crate) port: u16,
    pub(crate) password: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ListedContainer {
    name: String,
    image: String,
    status: String,
    web_port: Option<u16>,
    web_password: Option<String>,
    host_ports: BTreeSet<u16>,
}

#[derive(Debug, Clone)]
struct ContainerRow {
    name: String,
    image: String,
    status: String,
    labels: String,
    ports: String,
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
    let web = if args.no_web {
        None
    } else {
        Some(WebSettings {
            port: allocate_web_port()?,
            password: container_web_password(&name),
        })
    };
    let podman_args = build_podman_run_args(
        &name,
        args.no_shared_auth,
        &args.mounts,
        &runtime_image.image,
        web.as_ref(),
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
    let no_web = target.no_web;
    let name = resolve_existing_container(target)?;
    if no_web {
        run_interactive("podman", vec!["attach".into(), name.into()])
    } else if let Some(web_port) = container_web_port_for_container(&name)? {
        wait_for_tcp_port(web_port, Duration::from_secs(30))?;
        open_url_in_browser(&format!("http://127.0.0.1:{web_port}"))
    } else {
        run_interactive("podman", vec!["attach".into(), name.into()])
    }
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
    let mut containers = query_agent_container_rows()?
        .into_iter()
        .map(|row| {
            let ContainerRow {
                name,
                image,
                status,
                labels,
                ports,
            } = row;
            let web_port = parse_web_port_label(&name, &labels)?;
            let container = ListedContainer {
                name,
                image,
                status,
                web_port,
                web_password: None,
                host_ports: parse_container_ports(&ports),
            };

            Ok(container.with_web(web_port))
        })
        .collect::<AppResult<Vec<_>>>()?;

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
    let port_width = containers
        .iter()
        .map(|container| container.web_port.map_or(4, |port| port.to_string().len()))
        .max()
        .unwrap_or(4)
        .max(4);
    let password_width = containers
        .iter()
        .map(|container| {
            container
                .web_password
                .as_ref()
                .map_or(8, |password| password.len())
        })
        .max()
        .unwrap_or(8)
        .max(8);

    println!(
        "{:<name_width$}  {:<image_width$}  {:<port_width$}  {:<password_width$}  STATUS",
        "NAME",
        "IMAGE",
        "PORT",
        "PASSWORD",
        name_width = name_width,
        image_width = image_width,
        port_width = port_width,
        password_width = password_width,
    );
    for container in containers {
        println!(
            "{:<name_width$}  {:<image_width$}  {:<port_width$}  {:<password_width$}  {}",
            container.name,
            container.image,
            container
                .web_port
                .map_or_else(|| "-".to_string(), |port| port.to_string()),
            container
                .web_password
                .clone()
                .unwrap_or_else(|| "-".to_string()),
            container.status,
            name_width = name_width,
            image_width = image_width,
            port_width = port_width,
            password_width = password_width,
        );
    }
}

impl ListedContainer {
    fn with_web(mut self, web_port: Option<u16>) -> Self {
        self.web_port = web_port;
        self.web_password = web_port.map(|_| container_web_password(&self.name));
        self
    }
}

fn container_web_port_for_container(name: &str) -> AppResult<Option<u16>> {
    let row = query_agent_container_rows()?
        .into_iter()
        .find(|container| container.name == name);

    match row {
        Some(row) => parse_web_port_label(&row.name, &row.labels),
        None => Ok(None),
    }
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
    let mut names = query_agent_container_rows()?
        .into_iter()
        .map(|row| row.name)
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
            if args.no_shared_auth || args.no_web || !args.mounts.is_empty() {
                return Err(AppError::Message(format!(
                    "container `{name}` already exists; create-only options only apply when creating a new container"
                )));
            }
            return Ok(StartPlan::Resume { name });
        }
        return Ok(StartPlan::Create(args));
    }

    if args.no_shared_auth || args.no_web || !args.mounts.is_empty() {
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

fn allocate_web_port() -> AppResult<u16> {
    let reserved_ports = reserved_web_ports()?;
    pick_web_port(&reserved_ports, port_is_available)
}

pub(crate) fn container_web_password(container_name: &str) -> String {
    container_name
        .strip_prefix(CONTAINER_PREFIX)
        .unwrap_or(container_name)
        .to_string()
}

pub(crate) fn pick_web_port<F>(reserved_ports: &BTreeSet<u16>, is_available: F) -> AppResult<u16>
where
    F: Fn(u16) -> bool,
{
    for port in WEB_PORT_START..=u16::MAX {
        if reserved_ports.contains(&port) {
            continue;
        }

        if is_available(port) {
            return Ok(port);
        }
    }

    Err(AppError::Message(
        "no free OpenCode web ports are available starting at 35080".to_string(),
    ))
}

fn reserved_web_ports() -> AppResult<BTreeSet<u16>> {
    let mut ports = BTreeSet::new();

    for container in load_agent_containers()? {
        if let Some(port) = container.web_port {
            ports.insert(port);
        }

        ports.extend(container.host_ports);
    }

    Ok(ports)
}

fn query_agent_container_rows() -> AppResult<Vec<ContainerRow>> {
    let filter = format!("name=^{}", CONTAINER_PREFIX);
    let output = run_and_capture(
        "podman",
        [
            "ps",
            "--all",
            "--filter",
            filter.as_str(),
            "--format",
            "{{.Names}}\t{{.Image}}\t{{.Status}}\t{{.Labels}}\t{{.Ports}}",
        ],
    )?;

    Ok(output.lines().filter_map(parse_container_row).collect())
}

fn parse_container_row(line: &str) -> Option<ContainerRow> {
    if line.trim().is_empty() {
        return None;
    }

    let mut fields = line.splitn(5, '\t');
    Some(ContainerRow {
        name: fields.next()?.trim().to_string(),
        image: fields.next()?.trim().to_string(),
        status: fields.next()?.trim().to_string(),
        labels: fields.next()?.trim().to_string(),
        ports: fields.next()?.trim().to_string(),
    })
}

fn parse_web_port_label(name: &str, labels: &str) -> AppResult<Option<u16>> {
    let labels = labels.trim();

    if labels.is_empty() || labels == "<none>" {
        return Ok(None);
    }

    for label in labels.split(|ch: char| ch == ',' || ch.is_whitespace()) {
        let label = label.trim();
        if label.is_empty() {
            continue;
        }

        let Some(value) = label
            .strip_prefix(CONTAINER_WEB_PORT_LABEL)
            .and_then(|value| value.strip_prefix('='))
        else {
            continue;
        };

        let port = value.parse::<u16>().map_err(|_| {
            AppError::Message(format!(
                "container `{name}` has an invalid OpenCode web port label `{value}`"
            ))
        })?;
        return Ok(Some(port));
    }

    Ok(None)
}

fn parse_container_ports(value: &str) -> BTreeSet<u16> {
    value
        .split(',')
        .filter_map(|binding| {
            let binding = binding.trim();
            if binding.is_empty() || binding == "<none>" {
                return None;
            }

            let (host, _) = binding.split_once("->")?;
            let host_port = host
                .trim()
                .rsplit_once(':')
                .map_or(host.trim(), |(_, port)| port.trim());
            host_port.parse::<u16>().ok()
        })
        .collect()
}

fn port_is_available(port: u16) -> bool {
    TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)).is_ok()
}

fn wait_for_tcp_port(port: u16, timeout: Duration) -> AppResult<()> {
    let deadline = Instant::now() + timeout;
    let addr = std::net::SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, port));

    loop {
        if TcpStream::connect_timeout(&addr, Duration::from_millis(100)).is_ok() {
            return Ok(());
        }

        if Instant::now() >= deadline {
            return Err(AppError::Message(format!(
                "OpenCode web did not start listening on port {port} within {}s",
                timeout.as_secs()
            )));
        }

        std::thread::sleep(Duration::from_millis(200));
    }
}

fn open_url_in_browser(url: &str) -> AppResult<()> {
    let candidates: &[(&str, &[&str])] = &[
        ("xdg-open", &[url] as &[&str]),
        ("gio", &["open", url] as &[&str]),
        ("sensible-browser", &[url] as &[&str]),
        ("open", &[url] as &[&str]),
    ];

    for (program, args) in candidates {
        match Command::new(program).args(*args).status() {
            Ok(status) if status.success() => return Ok(()),
            Ok(_) => continue,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(source) => {
                return Err(AppError::Io {
                    context: format!("failed to execute `{program}`"),
                    source,
                });
            }
        }
    }

    Err(AppError::Message(format!(
        "could not open browser for `{url}`; tried xdg-open, gio open, sensible-browser, and open"
    )))
}

pub(crate) fn build_podman_run_args(
    name: &str,
    no_shared_auth: bool,
    mounts: &[MountSpec],
    image: &str,
    web: Option<&WebSettings>,
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

    if let Some(web) = web {
        args.extend([
            OsString::from("--label"),
            OsString::from(format!("{}={}", CONTAINER_WEB_PORT_LABEL, web.port)),
            OsString::from("--env"),
            OsString::from(format!("{}={}", CONTAINER_WEB_PORT_ENV, web.port)),
            OsString::from("--env"),
            OsString::from(format!("{}={}", CONTAINER_WEB_PASSWORD_ENV, web.password)),
            OsString::from("--publish"),
            OsString::from(format!("127.0.0.1:{}:{}", web.port, web.port)),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn parses_container_row_from_ps_output() {
        let row = parse_container_row(
            "rm4dev-agent-alpha\tlocalhost/rm4dev-agent:nix-fedora\tUp 2 minutes\torg.rm4dev.opencode.web-port=35080\t127.0.0.1:35080->35080/tcp",
        )
        .unwrap();

        assert_eq!(row.name, "rm4dev-agent-alpha");
        assert_eq!(row.image, "localhost/rm4dev-agent:nix-fedora");
        assert_eq!(row.status, "Up 2 minutes");
        assert_eq!(row.labels, "org.rm4dev.opencode.web-port=35080");
        assert_eq!(row.ports, "127.0.0.1:35080->35080/tcp");
    }

    #[test]
    fn parses_host_ports_from_ps_output() {
        let ports =
            parse_container_ports("127.0.0.1:35080->35080/tcp, [::1]:35081->35081/tcp, <none>");

        assert_eq!(ports, BTreeSet::from([35080, 35081]));
    }

    #[test]
    fn treats_blank_web_port_label_as_missing() {
        assert_eq!(
            parse_web_port_label("rm4dev-agent-alpha", "").unwrap(),
            None
        );
        assert_eq!(
            parse_web_port_label("rm4dev-agent-alpha", "<none>").unwrap(),
            None
        );
    }

    #[test]
    fn parses_web_port_label_from_labels_output() {
        assert_eq!(
            parse_web_port_label(
                "rm4dev-agent-alpha",
                "foo=bar,org.rm4dev.opencode.web-port=35080,baz=qux"
            )
            .unwrap(),
            Some(35080)
        );
    }
}
