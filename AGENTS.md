# Agents

## Workspace Structure
- `apps/agent`: Agent implementation.
- `apps/cli`: CLI interface.
- `apps/server`: Server implementation.
- `crates/api-types`: Shared API types.
- `crates/job-executor`: Job execution logic.

## Dependency Management
- Enforce the use of `dependency.workspace = true` in sub-crates (refer to root `Cargo.toml`).

## Commands
- Run tests: `cargo test`
- Run specific application: `cargo run -p <package_name>`
- Server operations:
  - Serve: `cargo run -p server -- serve`
  - Migrate: `cargo run -p server -- migrate`
  - Forge: `cargo run -p server -- forge <subcommand>`

## Commit Conventions
- Use conventional commits (e.g., `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`).
