# Skills Repository

This folder contains small, reusable "skills" (prompt snippets / SOPs / checklists) that backend agents or the GUI can load at runtime.

## Structure

- `skills/index.json`: machine-readable index
- `skills/**.md`: skill bodies (Markdown)

## Conventions

- Each skill has a stable `id` (e.g. `gui.egui_componentization`).
- Skill body should be concise and actionable.
- Do not store secrets (API keys, passwords) in skills.

## Loading (Rust)

Use `vas_core::backend::SkillRepository`.

- Default root is the current working directory.
- `index.json` is expected at `skills/index.json` under the root.
