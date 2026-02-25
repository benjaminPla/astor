# Contributing to astor

astor exists for one reason: Rust services behind a reverse proxy. Not Rust services that *are* a reverse proxy.

If your idea belongs in nginx config, it does not belong here.
If your idea makes astor better at being astor, welcome.

---

- Fork the repo, branch from `master`, open a PR against `master`.
- PR title: `<type>: <short description>` — types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`.
- `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt --check` must all pass.
- Add or update tests for anything you changed.
- Update `CHANGELOG.md` under `[Unreleased]`.
- New dependencies require prior discussion. "It would be convenient" is not a good reason.
- No `unsafe` unless there is genuinely no alternative and the reasoning is documented next to the code.
- No new public API without doc comments.
- Be respectful. Be direct. Be constructive.

---

## People

### Maintainer

**benjamin_pla** — [benjaminpla.dev@gmail.com](mailto:benjaminpla.dev@gmail.com) · [GitHub](https://github.com/benjaminPla)

### Contributors

<!-- - username · email · github · website -->
