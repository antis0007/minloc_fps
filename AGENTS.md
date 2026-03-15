# Codex development guidance

- Keep total LOC and file count low.
- Do not split files unless necessary.
- Prefer editing existing modules over introducing new ones.
- Keep systems straightforward and centralized.
- Avoid speculative architecture and “for later” abstractions.
- Prefer `cargo check` over `cargo run`.
- Avoid `cargo clean` unless explicitly requested.
- Make visible, playable progress each task.
- When adding FPS features, preserve future multiplayer compatibility.
- When in doubt, choose the simplest implementation that still feels like a real FPS.

## Architecture and delivery defaults

- Use the current architecture as the base unless a change clearly removes bugs or duplication.
- Prioritize gameplay correctness over micro-reducing lines.
- After each task, summarize changed files and remaining issues.
