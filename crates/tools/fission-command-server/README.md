# fission-command-server

Implementation crate for the `fission server` command group.

The public developer interface is the `fission` binary published by the
`cargo-fission` crate. This internal crate is split out so the CLI can keep
server-specific build, check, route listing, serve, and browser-artifact logic
separate from platform run/package commands.

`fission server` drives server-rendered Fission projects by invoking the
project's configured server entrypoint, building route-local browser worker and
island artifacts, running route checks, listing routes, and starting the local
server used during development.

Install the CLI with:

```sh
cargo install cargo-fission
```

Then use it as:

```sh
fission server check --project-dir .
fission server serve --project-dir .
fission server build --project-dir . --release
```

See <https://fission.rs> for the full server workflow and configuration
reference.
