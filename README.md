# termeme

`termeme` is a small Rust CLI for terminal sound notifications.

It is designed for shell-hook usage around long-running commands so you can hear when work finishes, fails, or hits a deploy-like command path without constantly watching the terminal.

## What it does

- Plays bundled `success`, `error`, and `deploy` sounds
- Embeds audio assets directly in the binary
- Writes local config to `~/.termeme/config.toml`
- Stays quiet in hook mode unless `TERMEME_DEBUG=1`
- Ships a `doctor` command for local diagnostics

## Platform support

Sound playback currently uses `afplay`, so actual playback is macOS-only.

- macOS: fully supported
- Linux and Windows: the CLI builds, but `play` returns an unsupported-platform error

## Install

### From a GitHub Release

GitHub Releases include native macOS tarballs:

- `termeme-<version>-x86_64-apple-darwin.tar.gz`
- `termeme-<version>-aarch64-apple-darwin.tar.gz`

Example install:

```bash
tar -xzf termeme-<version>-aarch64-apple-darwin.tar.gz
mv termeme /usr/local/bin/termeme
```

### From source

Build locally:

```bash
cargo build
```

Install from the current checkout:

```bash
cargo install --path .
```

Confirm the installed version:

```bash
termeme --version
```

## Quick start

Initialize the local sound and config directory:

```bash
termeme init
```

Play sounds directly:

```bash
termeme play success
termeme play error
termeme play deploy
```

Run diagnostics:

```bash
termeme doctor
```

Manually exercise hook behavior:

```bash
termeme hook --exit-code 0 --duration-ms 2400 --command "cargo test"
```

## Commands

`termeme play <preset>`
- Plays one preset immediately

`termeme hook --exit-code <code> --duration-ms <ms> --command "<cmd>"`
- Evaluates a finished command and chooses the right preset

`termeme init`
- Writes bundled WAV files and a default config into `~/.termeme`

`termeme doctor`
- Reports platform support, config location, and asset presence

## Shell integration

`termeme` is meant to be called from your shell’s post-command hook rather than used interactively all day.

The hook contract is:

- `--exit-code`: exit status of the command that just finished
- `--duration-ms`: runtime of that command in milliseconds
- `--command`: original command string

Example manual invocation:

```bash
termeme hook --exit-code 1 --duration-ms 3200 --command "pnpm deploy"
```

## Configuration

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

Config fields:

- `min_duration_ms`: minimum runtime before any sound is played
- `deploy_command_prefixes`: command prefixes that should map to the `deploy` preset

## Hook selection rules

Hook mode uses this decision order:

1. Ignore commands shorter than `min_duration_ms`
2. Use `deploy` if the command starts with a configured deploy prefix
3. Use `success` for exit code `0`
4. Use `error` for non-zero exit codes

If config loading, path resolution, or playback fails during hook execution, `termeme` exits quietly by default. Set `TERMEME_DEBUG=1` to print debug errors to stderr.

## Troubleshooting

`termeme doctor` is the first command to run when setup looks wrong.

Common checks:

- Verify the config exists at `~/.termeme/config.toml`
- Verify bundled sounds were written into `~/.termeme`
- Verify `afplay` is available on macOS
- Use `TERMEME_DEBUG=1` to surface hook-time failures

## Development

Run local checks:

```bash
cargo check --locked
cargo test --locked
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release --locked
```

Current module layout:

- `src/main.rs`: bootstrap only
- `src/cli.rs`: clap CLI definitions
- `src/app.rs`: top-level command execution
- `src/config.rs`: TOML config loading and defaults
- `src/hook.rs`: hook preset selection logic
- `src/sound.rs`: embedded assets, init, doctor, and playback

## Releases

Versioning is managed by `release-plz` and conventional commits.

- `fix:` bumps the patch version
- `feat:` bumps the minor version
- `feat!:` or `BREAKING CHANGE:` bumps the major version

Release flow:

1. A push to `main` runs `release-plz`
2. `release-plz` opens or updates a release PR
3. Merging that PR creates the git tag and GitHub Release
4. The release asset workflow builds macOS tarballs and checksum files and uploads them to the release

Required GitHub repository setting:

- In `Settings > Actions > General`, enable `Allow GitHub Actions to create and approve pull requests`

Local preview:

```bash
cargo install release-plz
release-plz release-pr --config release-plz.toml
```
