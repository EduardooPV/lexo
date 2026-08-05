# Contributing to Lexo

Thanks for considering a contribution. Lexo is a small, focused project — please keep
that in mind when proposing changes.

## Before you start

For anything beyond a small fix, open an issue first to discuss the change. This avoids
spending time on a pull request that doesn't fit the project's direction.

## Development setup

See the [Development](README.md#development) section of the README for prerequisites
and the local dev loop (`npm run dev`).

There is no test suite or JS lint/build config — the frontend is plain files served
as-is. Before opening a pull request, run the backend checks from `src-tauri/`:

```bash
cargo check
cargo clippy
cargo fmt
```

A pull request with `cargo clippy` warnings or unformatted Rust code will not be merged
as-is.

## Style

- Match the existing code style rather than introducing a new one.
- Keep changes scoped to what the issue or feature actually requires — avoid unrelated
  refactors in the same pull request.
- New commands added to the Rust backend must be registered in the
  `invoke_handler![...]` list in `src-tauri/src/lib.rs`.
- Every clickable UI element should use the `.btn` class system in `src/base.css`
  rather than a new bespoke button style.

## Pull requests

- Describe what changed and why, not just what.
- Reference the issue it addresses, if any.
- Keep the scope focused — one feature or fix per pull request.

## Reporting bugs

Open an issue with steps to reproduce, what you expected, and what happened instead.
Include your OS and, if relevant, whether you're on the Free or Pro DeepL API tier.

## Security issues

Do not open a public issue for a security vulnerability — see [SECURITY.md](SECURITY.md).
