# Contributing

Thank you for helping improve IPCalc GUI. The project is intentionally small and direct, so contributions should stay maintainable, testable, and clearly scoped.

## Development Workflow

1. Fork the repository and create a feature branch.
2. Install Rust stable.
3. Run `cargo run` to verify the GUI behavior.
4. Make your code or documentation changes.
5. Run `cargo fmt` and `cargo test` before submitting.
6. Open a pull request and describe the change, verification steps, and related issues.

## Code Guidelines

- Keep network calculation logic in `src/ipcalc.rs` and cover it with unit tests.
- Keep UI changes in `ui/app.slint` and prefer simple, predictable layouts.
- Error messages should clearly explain what input needs to be fixed.
- Avoid large dependencies unless they clearly reduce complexity or improve correctness.
- Keep the Windows experience working. Cross-platform changes should mention which platforms were verified.

## Commit Messages

Use concise, action-oriented commit messages, for example:

```text
Add IPv6 range validation
Fix IPv4 wildcard mask parsing
Update README build steps
```

## Reporting Issues

When opening an issue, include as much of the following as possible:

- Operating system version.
- Application version or commit.
- Input value.
- Expected result and actual result.
- A screenshot for UI issues, when available.
