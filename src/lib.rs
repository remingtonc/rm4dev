mod agent;
mod cli;
mod error;
mod image;
mod mounts;
mod naming;
mod process;

use crate::agent::{
    attach_container, create_container, enter_container, list_running_containers, precheck,
    remove_container, start_container, stop_container,
};
use crate::cli::{CliCommand, parse_cli, render_help};

pub fn run<I, S>(args: I) -> i32
where
    I: IntoIterator<Item = S>,
    S: Into<std::ffi::OsString> + Clone,
{
    match run_inner(args) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("{error}");
            if matches!(error, error::AppError::Usage(_)) {
                eprintln!();
                eprintln!("{}", render_help());
            }
            error.exit_code()
        }
    }
}

fn run_inner<I, S>(args: I) -> error::AppResult<()>
where
    I: IntoIterator<Item = S>,
    S: Into<std::ffi::OsString> + Clone,
{
    match parse_cli(args)? {
        CliCommand::AgentPrecheck => precheck(),
        CliCommand::AgentList => list_running_containers(),
        CliCommand::AgentNew(args) => create_container(args),
        CliCommand::AgentStart(args) => start_container(args),
        CliCommand::AgentStop(target) => stop_container(target),
        CliCommand::AgentRemove(target) => remove_container(target),
        CliCommand::AgentAttach(target) => attach_container(target),
        CliCommand::AgentEnter(target) => enter_container(target),
        CliCommand::ImageBuild(args) => image::image_build(args),
        CliCommand::ImageEnsure(args) => image::image_ensure(args),
    }
}

#[cfg(test)]
mod tests {
    use super::agent::{StartPlan, build_podman_run_args, plan_start};
    use super::cli::{
        CliCommand, CreateContainerArgs, ImageCommandArgs, parse_cli, parse_create_args,
    };
    use super::image::{DEFAULT_IMAGE, RuntimeImage, resolve_image_ref, runtime_image_from_env};
    use super::mounts::MountSpec;
    use super::naming::normalize_container_name;
    use super::process::render_os_args;

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

        let parsed = parse_create_args(&args, false).unwrap();
        assert_eq!(parsed.name.as_deref(), Some("rm4dev-agent-alpha"));
        assert!(!parsed.no_shared_auth);
        assert_eq!(parsed.mounts.len(), 1);
        assert_eq!(parsed.mounts[0].container, "/workspace");
    }

    #[test]
    fn treats_first_mount_as_mount_without_name() {
        let tempdir = std::env::temp_dir();
        let args = vec![format!("{}:/workspace", tempdir.display())];

        let parsed = parse_create_args(&args, false).unwrap();
        assert_eq!(parsed.name, None);
        assert!(!parsed.no_shared_auth);
        assert_eq!(parsed.mounts.len(), 1);
    }

    #[test]
    fn parses_no_shared_auth_flag_for_new_container() {
        let parsed = parse_cli(["rm4dev", "agent", "new", "--no-shared-auth", "alpha"]).unwrap();
        assert_eq!(
            parsed,
            CliCommand::AgentNew(CreateContainerArgs {
                name: Some("rm4dev-agent-alpha".to_string()),
                no_shared_auth: true,
                mounts: Vec::new(),
            })
        );
    }

    #[test]
    fn shared_auth_is_enabled_by_default() {
        let parsed = parse_cli(["rm4dev", "agent", "new", "alpha"]).unwrap();
        assert_eq!(
            parsed,
            CliCommand::AgentNew(CreateContainerArgs {
                name: Some("rm4dev-agent-alpha".to_string()),
                no_shared_auth: false,
                mounts: Vec::new(),
            })
        );
    }

    #[test]
    fn start_prefers_existing_named_container() {
        let plan = plan_start(
            vec!["rm4dev-agent-alpha".to_string()],
            CreateContainerArgs {
                name: Some("rm4dev-agent-alpha".to_string()),
                no_shared_auth: false,
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
            CreateContainerArgs {
                name: Some("rm4dev-agent-beta".to_string()),
                no_shared_auth: false,
                mounts: Vec::new(),
            },
        )
        .unwrap();

        assert_eq!(
            plan,
            StartPlan::Create(CreateContainerArgs {
                name: Some("rm4dev-agent-beta".to_string()),
                no_shared_auth: false,
                mounts: Vec::new(),
            })
        );
    }

    #[test]
    fn start_uses_mounts_as_create_signal_without_name() {
        let mount = fixture_mount(std::path::Path::new("/tmp"));
        let plan = plan_start(
            vec!["rm4dev-agent-alpha".to_string()],
            CreateContainerArgs {
                name: None,
                no_shared_auth: false,
                mounts: vec![mount.clone()],
            },
        )
        .unwrap();

        assert_eq!(
            plan,
            StartPlan::Create(CreateContainerArgs {
                name: None,
                no_shared_auth: false,
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
            CreateContainerArgs {
                name: None,
                no_shared_auth: false,
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
            CreateContainerArgs {
                name: Some("rm4dev-agent-alpha".to_string()),
                no_shared_auth: false,
                mounts: vec![mount],
            },
        )
        .unwrap_err();

        assert!(format!("{error}").contains("create-only options only apply"));
    }

    #[test]
    fn start_uses_no_shared_auth_as_create_signal_without_name() {
        let plan = plan_start(
            vec!["rm4dev-agent-alpha".to_string()],
            CreateContainerArgs {
                name: None,
                no_shared_auth: true,
                mounts: Vec::new(),
            },
        )
        .unwrap();

        assert_eq!(
            plan,
            StartPlan::Create(CreateContainerArgs {
                name: None,
                no_shared_auth: true,
                mounts: Vec::new(),
            })
        );
    }

    #[test]
    fn start_rejects_no_shared_auth_for_existing_named_container() {
        let error = plan_start(
            vec!["rm4dev-agent-alpha".to_string()],
            CreateContainerArgs {
                name: Some("rm4dev-agent-alpha".to_string()),
                no_shared_auth: true,
                mounts: Vec::new(),
            },
        )
        .unwrap_err();

        assert!(format!("{error}").contains("create-only options only apply"));
    }

    #[test]
    fn cpu_quota_reserves_a_quarter_of_cores() {
        assert!(super::agent::cpu_quota() >= 1);
    }

    #[test]
    fn build_run_args_contains_required_flags() {
        let mount = fixture_mount(std::path::Path::new("/tmp"));
        let args =
            build_podman_run_args("rm4dev-agent-alpha", false, &[mount], DEFAULT_IMAGE).unwrap();
        let rendered = render_os_args(&args);

        assert!(rendered.starts_with(&["run".to_string(), "--interactive".to_string()]));
        assert!(rendered.contains(&"--mount".to_string()));
        assert_eq!(rendered.last().map(String::as_str), Some(DEFAULT_IMAGE));
    }

    #[test]
    fn parses_image_build_without_custom_ref() {
        let parsed = parse_cli(["rm4dev", "image", "build"]).unwrap();
        assert_eq!(
            parsed,
            CliCommand::ImageBuild(ImageCommandArgs { image: None })
        );
    }

    #[test]
    fn parses_image_build_with_custom_ref() {
        let parsed = parse_cli(["rm4dev", "image", "build", "localhost/custom:dev"]).unwrap();
        assert_eq!(
            parsed,
            CliCommand::ImageBuild(ImageCommandArgs {
                image: Some("localhost/custom:dev".to_string()),
            })
        );
    }

    #[test]
    fn parses_image_ensure_with_custom_ref() {
        let parsed = parse_cli(["rm4dev", "image", "ensure", "quay.io/example/foo:tag"]).unwrap();
        assert_eq!(
            parsed,
            CliCommand::ImageEnsure(ImageCommandArgs {
                image: Some("quay.io/example/foo:tag".to_string()),
            })
        );
    }

    #[test]
    fn runtime_image_defaults_to_auto_managed_image() {
        assert_eq!(
            runtime_image_from_env(None),
            RuntimeImage {
                image: DEFAULT_IMAGE.to_string(),
                auto_ensure: true,
            }
        );
    }

    #[test]
    fn runtime_image_skips_auto_build_for_override() {
        assert_eq!(
            runtime_image_from_env(Some("localhost/custom:dev".to_string())),
            RuntimeImage {
                image: "localhost/custom:dev".to_string(),
                auto_ensure: false,
            }
        );
    }

    #[test]
    fn resolves_default_image_for_explicit_commands() {
        assert_eq!(resolve_image_ref(None), DEFAULT_IMAGE);
        assert_eq!(
            resolve_image_ref(Some("localhost/alt:tag")),
            "localhost/alt:tag"
        );
    }
}
