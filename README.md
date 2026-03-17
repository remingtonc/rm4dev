# rm4dev

`rm4dev` is a Rust CLI for managing `rm4dev-agent-*` Podman containers used for agent development.

## Commands

```text
rm4dev agent precheck
rm4dev agent list
rm4dev agent new [--no-shared-auth] [name] [host_path:container_path ...]
rm4dev agent start [--no-shared-auth] [name] [host_path:container_path ...]
rm4dev agent stop [name]
rm4dev agent rm [name]
rm4dev agent attach [name]
rm4dev agent enter [name]
rm4dev image build [image]
rm4dev image ensure [image]
```

## Behavior notes

- Names are normalized to `rm4dev-agent-<word>`. Passing `alpha` becomes `rm4dev-agent-alpha`.
- `start` resumes an existing container when it can resolve one unambiguously.
- `start` creates a new container when the chosen name does not already exist, when no containers exist, or when mount specs are supplied without a target name.
- `new` always creates a new container. If no name is provided, it generates one from the current Unix timestamp.
- Shared auth is enabled by default and binds the host file `~/.cache/rm4dev/opencode-auth.json` into `/root/.local/share/opencode/auth.json` so OpenCode auth persists across newly created containers.
- `--no-shared-auth` disables that default shared auth mount when creating a new container.
- `list` shows only running `rm4dev-agent-*` containers.
- `enter` opens a login shell inside a running container. Override the shell path with `RM4DEV_ENTER_SHELL`.

## Image configuration

By default, `rm4dev` runs the image `localhost/rm4dev-agent:nix-fedora`. Override that with:

```text
RM4DEV_IMAGE=my-image cargo run -- agent new
```

`rm4dev` can now build the bundled `nix-fedora` image directly from the binary, without a local `rm4dev-agent/` checkout.

- `rm4dev image build` builds the default image `localhost/rm4dev-agent:nix-fedora`.
- `rm4dev image build localhost/custom-agent:dev` builds the same embedded image under a custom tag.
- `rm4dev image ensure` only builds when the target image is missing locally.
- `rm4dev agent new` and `rm4dev agent start` automatically ensure the default image exists before creating a new container.
- When `RM4DEV_IMAGE` is set, image management becomes user-managed and implicit builds are skipped.

The embedded build still relies on network access for upstream dependencies such as the base image and package downloads.

`rm4dev` creates the host shared auth file on demand unless `--no-shared-auth` is used.
