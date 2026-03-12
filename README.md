# termeme

`termeme` is a small Rust CLI that plays sounds for terminal events.

It is aimed at shell-hook usage for long-running commands, with a few preset sounds:

- `success`
- `error`
- `deploy`

## Features

- Bundled audio assets embedded in the binary
- TOML-based configuration in `~/.termeme/config.toml`
- Safe shell-hook behavior that stays quiet unless `TERMEME_DEBUG=1`
- `doctor` command for local setup checks

## Install

Build locally:

```bash
cargo build
```

Install to your cargo bin directory:

```bash
cargo install --path .
```

Users can confirm the installed version with:

```bash
termeme --version
```

## Commands

Play a sound directly:

```bash
termeme play success
termeme play error
termeme play deploy
```

Initialize the local sound/config directory:

```bash
termeme init
```

Check local setup:

```bash
termeme doctor
```

Run the shell-hook entrypoint manually:

```bash
termeme hook --exit-code 0 --duration-ms 2400 --command "cargo test"
```

## Config

`termeme init` creates `~/.termeme/config.toml` if it does not already exist.

Default config:

```toml
min_duration_ms = 1500
deploy_command_prefixes = [
  "git push",
  "pnpm deploy",
  "npm publish",
  "vercel --prod",
]
```

### Config fields

- `min_duration_ms`: minimum command runtime before any sound is played
- `deploy_command_prefixes`: command prefixes that should use the `deploy` sound

## Hook behavior

The `hook` command chooses sounds using this order:

1. Ignore commands shorter than `min_duration_ms`
2. Use `deploy` if the command starts with a configured deploy prefix
3. Use `success` for exit code `0`
4. Use `error` for non-zero exit codes

If config loading, path resolution, or playback fails during hook execution, `termeme` exits quietly by default. Set `TERMEME_DEBUG=1` to print debug errors to stderr.

## Platform support

Playback currently uses `afplay`, so actual sound playback is macOS-only. The CLI still builds on other platforms, but `play` will return an unsupported-platform error and `doctor` will report that playback is unsupported.

## Development

Run checks:

```bash
cargo check
cargo test
```

## Releases

Versioning is managed by `release-plz` and driven by conventional commits.

- `fix:` bumps the patch version
- `feat:` bumps the minor version
- `feat!:` or a `BREAKING CHANGE:` footer bumps the major version

The release workflow runs on pushes to `main`:

1. `release-plz` opens or updates a release PR with the next version and changelog.
2. Merging that PR creates the git tag and GitHub release.
3. If `CARGO_REGISTRY_TOKEN` is configured in GitHub Actions secrets, the crate can also be published to crates.io.

Local preview commands:

```bash
cargo install release-plz
release-plz release-pr --config release-plz.toml
```

Current module layout:

- `src/main.rs`: bootstrap only
- `src/cli.rs`: clap CLI definitions
- `src/app.rs`: top-level command execution
- `src/config.rs`: TOML config loading/defaults
- `src/hook.rs`: hook sound-selection logic
- `src/sound.rs`: embedded assets, init, doctor, and playback
