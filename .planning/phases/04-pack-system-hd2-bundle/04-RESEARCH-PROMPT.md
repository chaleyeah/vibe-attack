<objective>
Research how to implement Phase 4: Pack System + HD2 Bundle
Answer: "What do I need to know to PLAN this phase well?"
</objective>

<files_to_read>
- .planning/phases/04-pack-system-hd2-bundle/04-CONTEXT.md (USER DECISIONS)
- .planning/REQUIREMENTS.md (Project requirements)
- .planning/STATE.md (Project decisions and history)
- src/config.rs (Existing config)
- src/main.rs (Daemon entry point)
</files_to_read>

<additional_context>
**Phase description**: Deliver a robust management system for macro "packs" and a complete Helldivers 2 stratagem bundle. Support for .hdpack files (Zip archives), profile management, and a built-in interactive TUI editor.
**Phase requirement IDs**: ACT-03, STT-03, PACK-01, PACK-02, PACK-03, PACK-04, PACK-05

**Specific Research Questions**:
1. Best Rust crate for ZIP management (is `zip` sufficient or `async-zip` needed?).
2. Implementation of a Unix Domain Socket control channel in a Tokio-based Rust daemon.
3. Best practices for a "Category Grouping" YAML schema that supports flattening for the matcher.
4. TUI library choice (Ratatui vs Cursive) for a "tactical/military" look and easy macro editing.
5. Checksum/Hashing best practices for pack verification.
</additional_context>

<output>
Write to: .planning/phases/04-pack-system-hd2-bundle/04-RESEARCH.md
</output>
