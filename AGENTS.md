# Agent Instructions

## Commit & Push

After completing any task that modifies files, **always commit and push** unless the user explicitly asks otherwise.

1. `git add` only the files you changed or created (do not stage unrelated untracked files).
2. Write a concise commit message in the repo's style.
3. `git commit`
4. `git push`

## Lint & Typecheck

For Rust changes in `rust/`, always run before committing:

```bash
cd rust
cargo fmt --check
cargo clippy --release -- -D warnings
cargo test
```

## Notes

- Do not commit secrets or keys.
- Do not stage untracked files that are not part of the current task.
- This project is pure Rust; the legacy Java code is for reference only.
