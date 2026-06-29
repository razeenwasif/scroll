# Scroll — Development Context

> Session handoff document for AI-assisted development continuity.

## Project Overview

Scroll is a self-hosted reading hub, web clipper, and spaced-repetition engine.
It replaces Readwise Reader, Pocket, Instapaper, and Omnivore.

**Architecture**: Hybrid Rust TUI (ratatui 0.29 + crossterm 0.28) with an embedded
Axum web server on port 3131.

## Current State

### Phase 1 — Foundation (IN PROGRESS)
- [x] Project scaffold (Cargo.toml, directory structure)
- [x] Shared data models (src/models.rs)
- [x] Configuration management (src/config.rs)
- [x] Database schema and migrations (migrations/001_initial.sql)
- [ ] Storage layer (src/storage/) — SQLite CRUD via rusqlite
- [ ] Engine: Web scraper (src/engine/scraper.rs)
- [ ] Engine: SM-2 algorithm (src/engine/sm2.rs)
- [ ] Engine: Text utilities (src/engine/markdown.rs)
- [ ] AI integration: Ollama client (src/ai/)
- [ ] Embedded web server: Clipper form + REST API (src/server/)
- [ ] TUI: Library view (src/ui/library.rs)
- [ ] TUI: Reader view (src/ui/reader.rs)
- [ ] TUI: Review view (src/ui/review.rs)
- [ ] TUI: Highlights view (src/ui/highlights.rs)
- [ ] TUI: Theme system (src/ui/theme.rs)
- [ ] App state + event loop (src/app.rs)
- [ ] CLI entry point (src/main.rs)

### Upcoming Phases
- Phase 2: Highlight creation workflow, full web server
- Phase 3: Spaced repetition polish, streak tracking
- Phase 4: AI summaries, flashcard generation, article Q&A
- Phase 5: Onyx export, command palette, theming, polish

## Key Design Decisions

1. **Hybrid architecture**: TUI is primary interface; Axum server runs in background
   for web clipping and PDF viewing.
2. **SQLite**: First project in the ecosystem to use a real database (justified by
   article/highlight/flashcard data volume). Stored at `~/.local/share/scroll/scroll.db`.
3. **Arc<Mutex<Connection>>**: Database handle is Clone + Send + Sync for sharing
   between TUI thread and Axum handlers.
4. **SM-2**: Standard SuperMemo 2 algorithm for flashcard scheduling.
5. **No external readability library**: Custom scraper using the `scraper` crate
   with heuristic content extraction.

## Module Map

```
src/
├── main.rs          — CLI (clap) + startup
├── app.rs           — App state + TUI event loop
├── config.rs        — ~/.config/scroll/config.toml
├── models.rs        — Shared data types
├── storage/         — SQLite CRUD (rusqlite)
├── engine/          — Scraper, SM-2, text utils
├── ai/              — Ollama client + prompts
├── server/          — Axum web server
└── ui/              — TUI views (ratatui)
```

## Dependencies on Other Projects

- **Onyx** (~/Onyx): Planned highlight export to Onyx vault markdown format
- **Ollama**: Required running locally for AI features (summaries, flashcards, chat)
