# Codex development guidance

- Prefer `cargo check` for normal validation.
- Do not run `cargo clean` unless explicitly requested.
- Avoid `cargo run` unless runtime behavior must be verified.
- Keep heavy changes scoped and minimize unnecessary rebuilds.
- Optimize for iterative Rust development with ChatGPT Codex, minimizing compile times on Codex compute.
