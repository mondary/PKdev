# PKdev — Agent Manager

Lightweight terminal dev environment for orchestrating AI agents.
Built around **Kaku** (WezTerm fork) + clickable **Kanban TUI** in
Rust + **opencode** as conductor, coordinated by **herdr**.

## Quick start

### 1. Install fonts

```bash
chmod +x scripts/install-fonts.sh
./scripts/install-fonts.sh
```

Installs: JetBrainsMono Nerd Font, FiraCode Nerd Font, Hack Nerd Font,
Inter, Poppins.

### 2. Build the Kanban

```bash
cd kanban && cargo build --release
```

Binary is in `kanban/target/release/kanban`.

### 3. Launch

**Option A — in Kaku (recommended)**: the full workspace (terminal +
kanban + herdr integration).

```bash
./start
```

`start` compiles the kanban if needed, closes previous instances and launches
Kaku with the project config (`kaku/workspace.lua`), cwd = project folder.

**Option B — kanban alone, in any terminal**:

```bash
cd /Users/clm/Documents/GitHub/PROJECTS/PKdev
./kanban/target/release/kanban
```

On first launch, `tickets.db` is created automatically (full schema +
demo data). The kanban must be launched **from the project folder**
to find the DB and resolve project paths.

## Overview

The app starts on the **Projects** screen. Each project points to a folder
(cwd); opening it enters the **board** (4 columns: backlog / doing /
review / done). Each ticket opens a **detail view** from which git branches
and opencode agents are launched.

### Projects

| Key | Action |
|-----|--------|
| `Enter` / click | Open selected project |
| `a` | Add project — **TUI folder browser** |
| `d` | Delete (confirm `y/n`) |
| `Tab` | Toggle grid ⇄ list view |
| `← →` / `h l` | Navigate between projects |
| `j/k` | Navigate (list view) |
| `t` | Change theme |
| `?` | Help |

#### Folder browser (TUI)

Opens on `a`. 100% terminal, no GUI dependencies.

| Key | Action |
|-----|--------|
| `j/k` / `↑ ↓` | Navigate |
| `Enter` / `→` | Open folder, or confirm "✓ Use this folder" |
| `h` / `←` / `Backspace` | Parent folder |
| `n` | **Create new folder** (enter name, `Enter` = create) |
| `Esc` | Cancel |

> **Auto-repair**: if a project path no longer exists (folder renamed/moved),
> the kanban automatically resets it to the current cwd as long as it's a git
> repo. No need to delete/recreate.

### Board (tickets)

| Key | Action |
|-----|--------|
| `h j k l` / arrows | Navigate between columns and tickets |
| `H` / `L` | Move ticket left/right |
| `a` | Add ticket |
| `d` | Delete ticket |
| `Enter` / click | Open detail view |
| right click | Context menu (detail / move / delete) |
| drag & drop | Move between columns |
| `p` / `Esc` | Back to projects |
| `r` | Reload from DB |
| `t` / `?` | Theme / Help |

### Ticket detail view

| Key | Action |
|-----|--------|
| type + `Enter` | Send prompt to conductor |
| `Ctrl+F` | Focus on conductor agent |
| `Ctrl+U` | Clear input |
| `Esc` | Back to board |

## herdr integration

The kanban pilots herdr (`HERDR_BIN_PATH` or `herdr` in the `PATH`) to:

- list agents (`herdr agent list --json`) — status indicator on each card (● working / blocked / idle),
- start an agent per ticket,
- focus + send prompt to conductor.

herdr exposes `HERDR_WORKSPACE_ID`, `HERDR_TAB_ID`, `HERDR_PANE_ID` in each
managed pane: the kanban uses them to target the right workspace when opening
agents.

## Kaku shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+Shift+T` | Toggle Dracula / Catppuccin |
| `Cmd+D` | Horizontal split |
| `Cmd+Shift+D` | Vertical split |
| `Cmd+Opt+←↑↓→` | Navigate between panes |
| `Cmd+W` | Close pane |
| `Cmd+T` | New tab |

## Project structure

```
PKdev/
├── PROJECT.md              Full spec (architecture, roadmap)
├── README.md               This file
├── start                   Kaku launcher (build + workspace)
├── kanban/                 Kanban TUI (Rust + ratatui)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         Entry + event loop + DB init
│       ├── app.rs          State + logic + events + browser
│       ├── db.rs           SQLite layer (CRUD)
│       ├── herdr.rs        Bridge to herdr (agents, focus)
│       ├── ui.rs           Ratatui rendering
│       └── theme.rs        Palettes Dracula + Catppuccin
├── kaku/
│   ├── workspace.lua       Kaku config (fonts, theme, shortcuts)
│   └── kaku.lua            Multi-pane layout reference
├── scripts/
│   ├── install-fonts.sh    Nerd Fonts + Inter + Poppins
│   └── schema.sql          SQLite schema + demo data
└── tickets.db              Created on first launch (gitignored)
```

## Themes

Two integrated palettes, switchable with `t`:

- **Catppuccin Mocha** (default) — soft, pastel
- **Dracula** — contrasted, neon

## What works

- Projects screen (grid + list view, `Tab`)
- TUI folder browser with folder creation
- Auto-repair of project paths (move/rename)
- 4-column board, keyboard + mouse navigation, drag & drop
- Detail view, prompts, git branches per ticket
- Launch opencode agent (herdr current workspace)
- SQLite persistence
- Dracula + Catppuccin themes

## Roadmap

See `PROJECT.md` for full details.

- MCP bridge (tickets.db <-> conductor opencode)
- Conductor sub-agents (spawn/validate)
- Auto-merge on validation
- Live worker logs