# rm4dev

`rm4dev` is a Rust CLI for managing `rm4dev-agent-*` Podman containers used for agent development.

## Commands

```text
rm4dev agent precheck
rm4dev agent list
rm4dev agent new [name] [host_path:container_path ...]
rm4dev agent start [name] [host_path:container_path ...]
rm4dev agent stop [name]
rm4dev agent rm [name]
rm4dev agent attach [name]
rm4dev agent enter [name]
```

## Behavior notes

- Names are normalized to `rm4dev-agent-<word>`. Passing `alpha` becomes `rm4dev-agent-alpha`.
- `start` resumes an existing container when it can resolve one unambiguously.
- `start` creates a new container when the chosen name does not already exist, when no containers exist, or when mount specs are supplied without a target name.
- `new` always creates a new container. If no name is provided, it generates one from the current Unix timestamp.
- `list` shows only running `rm4dev-agent-*` containers.
- `enter` opens a login shell inside a running container. Override the shell path with `RM4DEV_ENTER_SHELL`.

## Image configuration

By default, `rm4dev` runs the image `rm4dev-agent`. Override that with:

```text
RM4DEV_IMAGE=my-image cargo run -- agent new
```

note: opencode stores auth config in ~/.local/share/opencode/auth.json for chatgpt auth etc.