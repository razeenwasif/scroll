# Scroll

A self-hosted reading hub, web clipper, and spaced-repetition engine.

> Replaces: Readwise Reader, Pocket, Instapaper, Omnivore.

## Features

- **Library** — Unified view of all saved articles, PDFs, and notes with tags, filters, and fuzzy search
- **Web Clipper** — Paste a URL (TUI or web form), Scroll scrapes it into clean markdown
- **Reader** — Distraction-free markdown reader with reading progress tracking
- **Highlights** — Inline text highlighting with color coding and annotations
- **AI** — Local LLM integration via Ollama for summaries, flashcard generation, and Q&A
- **Spaced Repetition** — SM-2 algorithm with daily review sessions and streak tracking
- **Web Server** — Embedded Axum server on `:3131` for browser-based clipping and PDF viewing

## Install

```bash
cargo install --path .
```

## Usage

```bash
# Launch the TUI
scroll

# Clip a URL from the command line
scroll clip https://example.com/article

# Launch directly into review mode
scroll review

# Run the web server only (no TUI)
scroll serve
```

## Keybindings

### Global
| Key | Action |
| --- | ------ |
| `1` | Library view |
| `2` | Review view |
| `3` | Highlights view |
| `/` | Search |
| `:` | Command bar |
| `q` | Quit / back |

### Library
| Key | Action |
| --- | ------ |
| `j/k` | Navigate |
| `Enter` | Open article |
| `a` | Add article (URL) |
| `d` | Archive |
| `f` | Toggle favorite |
| `Tab` | Cycle filters |

### Reader
| Key | Action |
| --- | ------ |
| `j/k` | Scroll |
| `G` | Jump to bottom |
| `gg` | Jump to top |
| `h` | Create highlight |
| `q` | Back to library |

### Review
| Key | Action |
| --- | ------ |
| `Space` | Flip card |
| `1-5` | Rate difficulty |
| `s` | Skip |
| `q` | End session |

## Configuration

Config file: `~/.config/scroll/config.toml`

```toml
[general]
default_view = "library"

[theme]
accent = "cyan"    # cyan, green, purple, gold, rose
style = "dark"

[ai]
enabled = true
ollama_url = "http://localhost:11434"
model = "gemma2"

[server]
enabled = true
port = 3131

[review]
cards_per_day = 20

[export]
onyx_vault_path = ""
```

## Architecture

Hybrid Rust TUI + embedded web server. Single binary.

- **TUI**: ratatui 0.29 + crossterm 0.28 (vim-style keybindings)
- **Web Server**: Axum on `:3131` (web clipper + PDF viewer + REST API)
- **Database**: SQLite via rusqlite (stored at `~/.local/share/scroll/scroll.db`)
- **AI**: Local Ollama for summaries, flashcards, Q&A
- **Search**: SQLite FTS5 full-text search

## License

MIT
