use std::env;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const CONTAINER_PREFIX: &str = "rm4dev-agent-";
const DEFAULT_IMAGE: &str = "localhost/rm4dev-agent:nix-fedora";
const IMAGE_ENV: &str = "RM4DEV_IMAGE";
const USAGE: &str = "\
Usage:
  rm4dev agent precheck
  rm4dev agent list
  rm4dev agent new [name] [host_path:container_path ...]
  rm4dev agent start [name] [host_path:container_path ...]
  rm4dev agent stop [name]
  rm4dev agent rm [name]
  rm4dev agent attach [name]
  rm4dev agent enter [name]

Notes:
  Names are normalized to the form rm4dev-agent-<word>.
  start resumes an existing container when it can resolve one.
  start creates a new container when no existing match is chosen or mount specs are supplied.
  Configure the image with RM4DEV_IMAGE; default is rm4dev-agent.";

#[derive(Debug)]
pub enum AppError {
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
    fn exit_code(&self) -> i32 {
        match self {
            Self::Usage(_) => 2,
            Self::Message(_) | Self::Io { .. } | Self::CommandFailed { .. } => 1,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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

type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct TargetArgs {
    name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CreateArgs {
    name: Option<String>,
    mounts: Vec<MountSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DevAgentCommand {
    Precheck,
    List,
    New(CreateArgs),
    Start(CreateArgs),
    Stop(TargetArgs),
    Remove(TargetArgs),
    Attach(TargetArgs),
    Enter(TargetArgs),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MountSpec {
    host: PathBuf,
    container: String,
}

impl MountSpec {
    fn podman_mount_arg(&self) -> String {
        format!(
            "type=bind,src={},target={}",
            self.host.display(),
            self.container
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum StartPlan {
    Create(CreateArgs),
    Resume { name: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RunningContainer {
    name: String,
    image: String,
    status: String,
}

pub fn run<I, S>(args: I) -> i32
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    match run_inner(args) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("{error}");
            if matches!(error, AppError::Usage(_)) {
                eprintln!();
                eprintln!("{USAGE}");
            }
            error.exit_code()
        }
    }
}

fn run_inner<I, S>(args: I) -> AppResult<()>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    match parse_cli(args)? {
        DevAgentCommand::Precheck => precheck(),
        DevAgentCommand::List => list_running_containers(),
        DevAgentCommand::New(args) => create_container(args),
        DevAgentCommand::Start(args) => start_container(args),
        DevAgentCommand::Stop(target) => stop_container(target),
        DevAgentCommand::Remove(target) => remove_container(target),
        DevAgentCommand::Attach(target) => attach_container(target),
        DevAgentCommand::Enter(target) => enter_container(target),
    }
}

fn parse_cli<I, S>(args: I) -> AppResult<DevAgentCommand>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let args = args
        .into_iter()
        .map(Into::into)
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    if args.is_empty() {
        return Err(AppError::Usage("missing argv[0]".to_string()));
    }

    let rest = &args[1..];
    if rest.is_empty() || rest.iter().any(|arg| arg == "--help" || arg == "-h") {
        return Err(AppError::Usage("missing command".to_string()));
    }

    match rest {
        [scope, ..] if scope != "agent" => Err(AppError::Usage(format!(
            "unsupported command scope `{scope}`; only `agent` is implemented"
        ))),
        [_scope, subcommand, tail @ ..] => parse_agent_command(subcommand, tail),
        _ => Err(AppError::Usage("missing agent subcommand".to_string())),
    }
}

fn parse_agent_command(subcommand: &str, args: &[String]) -> AppResult<DevAgentCommand> {
    match subcommand {
        "precheck" => {
            ensure_no_extra_args(subcommand, args)?;
            Ok(DevAgentCommand::Precheck)
        }
        "list" => {
            ensure_no_extra_args(subcommand, args)?;
            Ok(DevAgentCommand::List)
        }
        "new" => Ok(DevAgentCommand::New(parse_create_args(args)?)),
        "start" => Ok(DevAgentCommand::Start(parse_create_args(args)?)),
        "stop" => Ok(DevAgentCommand::Stop(parse_target_args(subcommand, args)?)),
        "rm" => Ok(DevAgentCommand::Remove(parse_target_args(
            subcommand, args,
        )?)),
        "attach" => Ok(DevAgentCommand::Attach(parse_target_args(
            subcommand, args,
        )?)),
        "enter" => Ok(DevAgentCommand::Enter(parse_target_args(subcommand, args)?)),
        other => Err(AppError::Usage(format!(
            "unsupported agent subcommand `{other}`"
        ))),
    }
}

fn ensure_no_extra_args(subcommand: &str, args: &[String]) -> AppResult<()> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(AppError::Usage(format!(
            "`rm4dev agent {subcommand}` does not take extra arguments"
        )))
    }
}

fn parse_target_args(subcommand: &str, args: &[String]) -> AppResult<TargetArgs> {
    match args {
        [] => Ok(TargetArgs { name: None }),
        [name] => Ok(TargetArgs {
            name: Some(normalize_container_name(name)?),
        }),
        _ => Err(AppError::Usage(format!(
            "`rm4dev agent {subcommand}` accepts at most one container name"
        ))),
    }
}

fn parse_create_args(args: &[String]) -> AppResult<CreateArgs> {
    if args.is_empty() {
        return Ok(CreateArgs {
            name: None,
            mounts: Vec::new(),
        });
    }

    let (name, mount_args) = if looks_like_mount_spec(&args[0]) {
        (None, args)
    } else {
        (Some(normalize_container_name(&args[0])?), &args[1..])
    };

    let mounts = mount_args
        .iter()
        .map(|arg| parse_mount_spec(arg))
        .collect::<AppResult<Vec<_>>>()?;

    Ok(CreateArgs { name, mounts })
}

fn looks_like_mount_spec(candidate: &str) -> bool {
    candidate.rsplit_once(':').is_some()
}

fn parse_mount_spec(input: &str) -> AppResult<MountSpec> {
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

fn normalize_container_name(input: &str) -> AppResult<String> {
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

fn is_valid_container_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(ch) if ch.is_ascii_alphanumeric() => (),
        _ => return false,
    }

    chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
}

fn precheck() -> AppResult<()> {
    run_and_capture("podman", ["--version"])?;
    run_and_capture("podman", ["info"])?;
    println!("podman is installed and runnable");
    Ok(())
}

fn list_running_containers() -> AppResult<()> {
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
        .filter_map(parse_running_container_line)
        .filter(|container| is_agent_container_name(&container.name))
        .collect::<Vec<_>>();

    containers.sort_by(|left, right| left.name.cmp(&right.name));

    if containers.is_empty() {
        println!("No running rm4dev agent containers.");
        return Ok(());
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

    Ok(())
}

fn parse_running_container_line(line: &str) -> Option<RunningContainer> {
    if line.trim().is_empty() {
        return None;
    }

    let mut fields = line.splitn(3, '\t');
    let name = fields.next()?.trim().to_string();
    let image = fields.next()?.trim().to_string();
    let status = fields.next()?.trim().to_string();

    Some(RunningContainer {
        name,
        image,
        status,
    })
}

fn create_container(args: CreateArgs) -> AppResult<()> {
    let name = args.name.unwrap_or_else(generate_container_name);
    ensure_container_does_not_exist(&name)?;
    let podman_args = build_run_args(&name, &args.mounts)?;
    run_interactive("podman", podman_args)
}

fn start_container(args: CreateArgs) -> AppResult<()> {
    let existing = list_agent_container_names()?;
    match plan_start(existing, args)? {
        StartPlan::Create(args) => create_container(args),
        StartPlan::Resume { name } => resume_container(&name),
    }
}

fn stop_container(target: TargetArgs) -> AppResult<()> {
    let name = resolve_existing_container(target)?;
    run_interactive("podman", vec!["stop".into(), name.into()])
}

fn remove_container(target: TargetArgs) -> AppResult<()> {
    let name = resolve_existing_container(target)?;
    run_interactive("podman", vec!["rm".into(), name.into()])
}

fn attach_container(target: TargetArgs) -> AppResult<()> {
    let name = resolve_existing_container(target)?;
    run_interactive("podman", vec!["attach".into(), name.into()])
}

fn enter_container(target: TargetArgs) -> AppResult<()> {
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
            "-l".into(),
        ],
    )
}

fn resolve_existing_container(target: TargetArgs) -> AppResult<String> {
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

fn is_agent_container_name(name: &str) -> bool {
    name.starts_with(CONTAINER_PREFIX)
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

fn plan_start(existing: Vec<String>, args: CreateArgs) -> AppResult<StartPlan> {
    if let Some(name) = args.name.clone() {
        if existing.iter().any(|existing_name| existing_name == &name) {
            if !args.mounts.is_empty() {
                return Err(AppError::Message(format!(
                    "container `{name}` already exists; mount specs only apply when creating a new container"
                )));
            }
            return Ok(StartPlan::Resume { name });
        }
        return Ok(StartPlan::Create(args));
    }

    if !args.mounts.is_empty() {
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

fn build_run_args(name: &str, mounts: &[MountSpec]) -> AppResult<Vec<OsString>> {
    let image = env::var(IMAGE_ENV).unwrap_or_else(|_| DEFAULT_IMAGE.to_string());
    let cpus = cpu_quota();

    let mut args = vec![
        OsString::from("run"),
        OsString::from("--interactive"),
        OsString::from("--tty"),
        OsString::from("--name"),
        OsString::from(name),
        OsString::from("--security-opt"),
        OsString::from("label=disable"),
        OsString::from("--device"),
        OsString::from("/dev/fuse"),
        OsString::from("--cpus"),
        OsString::from(cpus.to_string()),
        OsString::from("--mount"),
        OsString::from("type=tmpfs,target=/tmp"),
    ];

    for mount in mounts {
        args.extend([
            OsString::from("--mount"),
            OsString::from(mount.podman_mount_arg()),
        ]);
    }

    args.push(OsString::from(image));
    Ok(args)
}

fn cpu_quota() -> usize {
    let total = std::thread::available_parallelism()
        .map(|parallelism| parallelism.get())
        .unwrap_or(1);
    let reserved = total / 4;
    total.saturating_sub(reserved).max(1)
}

fn generate_container_name() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{CONTAINER_PREFIX}{seconds}")
}

fn run_and_capture<I, S>(program: &str, args: I) -> AppResult<String>
where
    I: IntoIterator<Item = S> + Clone,
    S: AsRef<OsStr>,
{
    let args_vec = args
        .clone()
        .into_iter()
        .map(|arg| arg.as_ref().to_os_string())
        .collect::<Vec<_>>();

    let output = Command::new(program)
        .args(&args_vec)
        .output()
        .map_err(|source| AppError::Io {
            context: format!("failed to execute `{program}`"),
            source,
        })?;

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

fn run_interactive(program: &str, args: Vec<OsString>) -> AppResult<()> {
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

fn render_os_args(args: &[OsString]) -> Vec<String> {
    args.iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_mount(path: &std::path::Path) -> MountSpec {
        MountSpec {
            host: path.to_path_buf(),
            container: "/workspace".to_string(),
        }
    }

    #[test]
    fn normalizes_short_container_names() {
        assert_eq!(
            normalize_container_name("alpha").unwrap(),
            "rm4dev-agent-alpha"
        );
    }

    #[test]
    fn preserves_prefixed_container_names() {
        assert_eq!(
            normalize_container_name("rm4dev-agent-alpha").unwrap(),
            "rm4dev-agent-alpha"
        );
    }

    #[test]
    fn rejects_invalid_container_names() {
        let error = normalize_container_name("alpha/beta").unwrap_err();
        assert!(format!("{error}").contains("invalid container name"));
    }

    #[test]
    fn parses_name_and_mounts() {
        let tempdir = std::env::temp_dir();
        let args = vec![
            "alpha".to_string(),
            format!("{}:/workspace", tempdir.display()),
        ];

        let parsed = parse_create_args(&args).unwrap();
        assert_eq!(parsed.name.as_deref(), Some("rm4dev-agent-alpha"));
        assert_eq!(parsed.mounts.len(), 1);
        assert_eq!(parsed.mounts[0].container, "/workspace");
    }

    #[test]
    fn treats_first_mount_as_mount_without_name() {
        let tempdir = std::env::temp_dir();
        let args = vec![format!("{}:/workspace", tempdir.display())];

        let parsed = parse_create_args(&args).unwrap();
        assert_eq!(parsed.name, None);
        assert_eq!(parsed.mounts.len(), 1);
    }

    #[test]
    fn start_prefers_existing_named_container() {
        let plan = plan_start(
            vec!["rm4dev-agent-alpha".to_string()],
            CreateArgs {
                name: Some("rm4dev-agent-alpha".to_string()),
                mounts: Vec::new(),
            },
        )
        .unwrap();

        assert_eq!(
            plan,
            StartPlan::Resume {
                name: "rm4dev-agent-alpha".to_string()
            }
        );
    }

    #[test]
    fn start_creates_new_named_container_when_name_is_unused() {
        let plan = plan_start(
            vec!["rm4dev-agent-alpha".to_string()],
            CreateArgs {
                name: Some("rm4dev-agent-beta".to_string()),
                mounts: Vec::new(),
            },
        )
        .unwrap();

        assert_eq!(
            plan,
            StartPlan::Create(CreateArgs {
                name: Some("rm4dev-agent-beta".to_string()),
                mounts: Vec::new(),
            })
        );
    }

    #[test]
    fn start_uses_mounts_as_create_signal_without_name() {
        let mount = fixture_mount(std::path::Path::new("/tmp"));
        let plan = plan_start(
            vec!["rm4dev-agent-alpha".to_string()],
            CreateArgs {
                name: None,
                mounts: vec![mount.clone()],
            },
        )
        .unwrap();

        assert_eq!(
            plan,
            StartPlan::Create(CreateArgs {
                name: None,
                mounts: vec![mount],
            })
        );
    }

    #[test]
    fn start_requires_name_when_multiple_existing_containers_exist() {
        let error = plan_start(
            vec![
                "rm4dev-agent-alpha".to_string(),
                "rm4dev-agent-beta".to_string(),
            ],
            CreateArgs {
                name: None,
                mounts: Vec::new(),
            },
        )
        .unwrap_err();

        assert!(format!("{error}").contains("multiple rm4dev agent containers"));
    }

    #[test]
    fn start_rejects_mounts_for_existing_named_container() {
        let mount = fixture_mount(std::path::Path::new("/tmp"));
        let error = plan_start(
            vec!["rm4dev-agent-alpha".to_string()],
            CreateArgs {
                name: Some("rm4dev-agent-alpha".to_string()),
                mounts: vec![mount],
            },
        )
        .unwrap_err();

        assert!(format!("{error}").contains("mount specs only apply"));
    }

    #[test]
    fn cpu_quota_reserves_a_quarter_of_cores() {
        assert!(cpu_quota() >= 1);
    }

    #[test]
    fn build_run_args_contains_required_flags() {
        let mount = fixture_mount(std::path::Path::new("/tmp"));
        let args = build_run_args("rm4dev-agent-alpha", &[mount]);

        match args {
            Ok(args) => {
                let rendered = render_os_args(&args);
                assert!(rendered.starts_with(&["run".to_string(), "--interactive".to_string()]));
                assert!(rendered.contains(&"--mount".to_string()));
            }
            Err(error) => {
                assert!(
                    format!("{error}").contains("failed to execute `id`"),
                    "unexpected build_run_args error: {error}"
                );
            }
        }
    }
}
