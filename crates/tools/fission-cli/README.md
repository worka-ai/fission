# fission-cli

`fission-cli` provides the first-party Fission project scaffolding commands:

- `fission init`
- `fission add-target`
- `cargo fission ...`

## Usage

Create a new app:

```sh
fission init my-app
```

Create a new app against a local Fission checkout:

```sh
fission init my-app --local-path /path/to/fission
```

Add more platform targets:

```sh
cargo fission add-target web ios android --project-dir my-app
```

## Current platform status

- `windows`, `macos`, `linux`: scaffolded and runnable through the generated desktop entrypoint
- `web`, `ios`, `android`: scaffolded only in the current branch; the corresponding shells are still in progress

The CLI writes platform state to `fission.toml` and creates `platforms/<target>/README.md` placeholders for targets that are not runnable yet.
