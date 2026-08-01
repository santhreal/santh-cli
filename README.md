# santh-cli

![status](https://img.shields.io/badge/status-beta-blue.svg)
![license](https://img.shields.io/badge/license-MIT-green.svg)

Shared command-line contract for Santh tools: global flags, exit codes, config
resolution, and a single entry point that every Santh binary routes through.

## What it does

`santh-cli` is the scaffolding layer every Santh CLI binary builds on. It gives
each tool a consistent surface without re-implementing the boilerplate:

- A `SanthCli` trait plus `santh_main()` entry point that parse arguments, apply
  global flags, and translate a tool's result into a stable process exit code.
- Canonical `GlobalFlags` (verbosity, output format, config path) shared by every
  tool, so `--log-level`, `--output`, and friends mean the same thing everywhere.
- `SanthExitCode` and `SanthError` types that map tool outcomes to documented,
  scriptable exit codes.
- Config resolution (`resolve_config`) and structured finding emission
  (`emit_finding`) helpers.

## Quick start

Add the dependency and route your binary's `main` through `santh_main`:

```rust
use santh_cli::{santh_main, GlobalFlags, SanthCli, SanthExitCode};

struct MyTool;

impl SanthCli for MyTool {
    type Subcommand = MySubcommand; // your `clap::Subcommand`

    fn tool_name() -> &'static str { "mytool" }
    fn tool_version() -> &'static str { env!("CARGO_PKG_VERSION") }
    fn tool_description() -> &'static str { "does the thing" }

    fn run(globals: GlobalFlags, subcommand: Self::Subcommand) -> std::process::ExitCode {
        // ... your logic ...
        SanthExitCode::Success.into()
    }
}

fn main() -> std::process::ExitCode {
    santh_main::<MyTool>()
}
```

## When to use / when not

Use `santh-cli` when you are writing a Santh tool with a subcommand-based CLI:
it gives you the shared flags, exit-code contract, and config plumbing for free,
and keeps every tool's interface consistent.

Do not reach for it for a flat (non-subcommand) CLI, a library with no binary, or
a one-off script. The `SanthCli` trait requires a `clap::Subcommand`; flat CLIs
should adopt the contract only when they gain subcommands.

## Compared to alternatives

- **Hand-rolled `clap` per tool**: works, but every tool then invents its own
  flag names, exit codes, and config loading, which drift apart over time.
  `santh-cli` centralizes that contract so it stays uniform and is fixed once.
- **`clap` derive alone**: covers argument parsing but not the exit-code
  contract, global-flag semantics, or config resolution that `santh-cli` adds on
  top.

## How it fits in Santh

`santh-cli` sits at the ingress edge of every Santh binary. Tools depend on it to
parse input and report results; it depends on nothing tool-specific, so it never
pulls domain logic upward. Conformance is enforced by `santh-conform`'s
`cli-contract` rule, which checks that each binary routes through
`santh_cli::santh_main` / `SanthCli`.

## Contributing

Changes must keep the CLI contract backwards compatible: never remove or rename a
public flag, exit code, or trait method without a deprecation path. Run
`cargo test` and `santh-conform check .` before opening a change.

## License

Licensed under the MIT License. See `LICENSE-MIT` for details.
