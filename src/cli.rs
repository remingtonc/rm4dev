use crate::error::{AppError, AppResult};
use crate::mounts::{MountSpec, looks_like_mount_spec, parse_mount_spec};
use crate::naming::normalize_container_name;
use clap::{Args, CommandFactory, Parser, Subcommand, error::ErrorKind};
use std::ffi::OsString;

const AFTER_HELP: &str = "Names are normalized to the form rm4dev-agent-<word>.\nstart resumes an existing container when it can resolve one.\nstart creates a new container when no existing match is chosen or mount specs are supplied.\nConfigure the runtime image with RM4DEV_IMAGE; default is localhost/rm4dev-agent:nix-fedora.\nimage build and image ensure accept an optional custom image reference.";

#[derive(Debug, Parser)]
#[command(name = "rm4dev", about = "Manage rm4dev Podman containers", after_help = AFTER_HELP)]
struct CliArgs {
    #[command(subcommand)]
    scope: ScopeCommand,
}

#[derive(Debug, Subcommand)]
enum ScopeCommand {
    Agent(AgentArgs),
    Image(ImageArgs),
}

#[derive(Debug, Args)]
struct AgentArgs {
    #[command(subcommand)]
    command: AgentCommand,
}

#[derive(Debug, Subcommand)]
enum AgentCommand {
    Precheck,
    List,
    New(CreateCommandArgs),
    Start(CreateCommandArgs),
    Stop(TargetCommandArgs),
    Rm(TargetCommandArgs),
    Attach(TargetCommandArgs),
    Enter(TargetCommandArgs),
}

#[derive(Debug, Args)]
struct ImageArgs {
    #[command(subcommand)]
    command: ImageCommand,
}

#[derive(Debug, Subcommand)]
enum ImageCommand {
    Build(ImageRefArg),
    Ensure(ImageRefArg),
}

#[derive(Debug, Args)]
struct CreateCommandArgs {
    #[arg(value_name = "NAME_OR_MOUNT", num_args = 0..)]
    args: Vec<String>,
}

#[derive(Debug, Args)]
struct TargetCommandArgs {
    #[arg(value_name = "NAME")]
    name: Option<String>,
}

#[derive(Debug, Args)]
struct ImageRefArg {
    #[arg(value_name = "IMAGE")]
    image: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContainerTarget {
    pub(crate) name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CreateContainerArgs {
    pub(crate) name: Option<String>,
    pub(crate) mounts: Vec<MountSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImageCommandArgs {
    pub(crate) image: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CliCommand {
    AgentPrecheck,
    AgentList,
    AgentNew(CreateContainerArgs),
    AgentStart(CreateContainerArgs),
    AgentStop(ContainerTarget),
    AgentRemove(ContainerTarget),
    AgentAttach(ContainerTarget),
    AgentEnter(ContainerTarget),
    ImageBuild(ImageCommandArgs),
    ImageEnsure(ImageCommandArgs),
}

pub(crate) fn parse_cli<I, S>(args: I) -> AppResult<CliCommand>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString> + Clone,
{
    let parsed = CliArgs::try_parse_from(args).map_err(map_clap_error)?;

    match parsed.scope {
        ScopeCommand::Agent(agent) => match agent.command {
            AgentCommand::Precheck => Ok(CliCommand::AgentPrecheck),
            AgentCommand::List => Ok(CliCommand::AgentList),
            AgentCommand::New(args) => Ok(CliCommand::AgentNew(parse_create_args(&args.args)?)),
            AgentCommand::Start(args) => Ok(CliCommand::AgentStart(parse_create_args(&args.args)?)),
            AgentCommand::Stop(args) => Ok(CliCommand::AgentStop(parse_target_args(args.name)?)),
            AgentCommand::Rm(args) => Ok(CliCommand::AgentRemove(parse_target_args(args.name)?)),
            AgentCommand::Attach(args) => {
                Ok(CliCommand::AgentAttach(parse_target_args(args.name)?))
            }
            AgentCommand::Enter(args) => Ok(CliCommand::AgentEnter(parse_target_args(args.name)?)),
        },
        ScopeCommand::Image(image) => match image.command {
            ImageCommand::Build(args) => Ok(CliCommand::ImageBuild(parse_image_args(args.image)?)),
            ImageCommand::Ensure(args) => {
                Ok(CliCommand::ImageEnsure(parse_image_args(args.image)?))
            }
        },
    }
}

pub(crate) fn render_help() -> String {
    let mut command = CliArgs::command();
    command.render_help().to_string()
}

fn map_clap_error(error: clap::Error) -> AppError {
    let exit_code = match error.kind() {
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => 0,
        _ => 2,
    };

    AppError::Cli {
        message: error.render().to_string(),
        exit_code,
    }
}

fn parse_image_args(image: Option<String>) -> AppResult<ImageCommandArgs> {
    match image {
        None => Ok(ImageCommandArgs { image: None }),
        Some(image) => {
            let image = image.trim();
            if image.is_empty() {
                Err(AppError::Usage(
                    "image reference must not be empty".to_string(),
                ))
            } else {
                Ok(ImageCommandArgs {
                    image: Some(image.to_string()),
                })
            }
        }
    }
}

fn parse_target_args(name: Option<String>) -> AppResult<ContainerTarget> {
    Ok(ContainerTarget {
        name: name.as_deref().map(normalize_container_name).transpose()?,
    })
}

pub(crate) fn parse_create_args(args: &[String]) -> AppResult<CreateContainerArgs> {
    if args.is_empty() {
        return Ok(CreateContainerArgs {
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

    Ok(CreateContainerArgs { name, mounts })
}
