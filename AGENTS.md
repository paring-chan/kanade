# Agents

## Workspace Structure
- `apps/agent`: Agent implementation.
- `apps/cli`: CLI interface.
- `apps/server`: Server implementation.

## Dependency Management
- Enforce the use of `dependency.workspace = true` in sub-crates (refer to root `Cargo.toml`).

## Commands
- Use `cargo test` to run tests.
- Use `cargo run -p <package_name>` to run specific applications.
