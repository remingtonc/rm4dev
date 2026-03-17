<!-- Copyright (C) 2026 RM4 LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Contributing

Thanks for improving `rm4dev`.

## Development setup

1. Install a working Rust toolchain and `cargo`.
2. Install Podman and confirm `podman info` succeeds for your user.
3. Clone the repository and build the project:

```text
cargo build
cargo test
```

## Common workflows

- `cargo fmt` keeps Rust files formatted.
- `cargo test` covers CLI parsing and core decision logic.
- `cargo run -- agent new demo` exercises the default image path.
- `cargo run -- image build` rebuilds the embedded `nix-fedora` image locally.
- `./build.sh` builds both image definitions under `rm4dev-agent/` and then the release binary.

## When changing behavior

- Keep `README.md` and the files under `docs/` aligned with user-visible behavior.
- Update tests when you change CLI parsing, naming, mount parsing, or container lifecycle decisions.
- Prefer documenting security-relevant defaults whenever you touch mounts, auth persistence, container privileges, or image contents.

## Contribution provenance and licensing

- This project is licensed under `GPL-3.0-or-later`.
- By submitting a contribution, you represent that you have the right to contribute the change under that license.
- Please preserve existing copyright notices and SPDX identifiers.
- If a contribution introduces third-party code or content, include its provenance and license details in the pull request.
- Maintainers may request additional provenance or assignment paperwork before merging substantial third-party contributions.

## Review checklist

- The change is documented where users or operators would expect to find it.
- Tests were added or updated when behavior changed.
- New files carry the correct copyright and SPDX notice.
- The change does not silently broaden security exposure or distribution obligations.
