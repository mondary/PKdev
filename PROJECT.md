# Gestionnaire d'Agents — Spec Projet

## Résumé

Un environnement de développement léger, 100% terminal, pour orchestrer des
agents IA (opencode) sur de multiples projets. Construit autour de **Kaku**
(fork WezTerm) comme socle, avec un Kanban TUI clickable, un conductor agent
qui délègue des sous-agents par ticket, et un switcher de projets pour
basculer entre <100 projets en gardant chaque session LLM vivante.

---

## Contexte et problème

**État actuel :**
- VSCode avec instances Opencode / Codex / Claude en parallèle
- Chaque instance VSCode plombe la RAM + CPU
- Chaque instance opencode consomme ~1 Go RAM
- Test en terminal (Kaku) + opencode : même problème de RAM par instance
- Objectif : Mac M5, le plus léger possible

**Causes identifiées :**
1. Le bottleneck n'est PAS le terminal ni le multiplexer — ce sont les
   processus agents eux-mêmes (Node.js + contexte LLM).
2. Lancer une instance opencode complète par tâche/ticket = explosion de RAM.

---

## Objectifs

1. **Légèreté** : ~1,5 Go RAM pour tout l'environnement (vs 5-10 Go actuels).
2. **Persistance** : revenir sur un projet et retrouver sa session LLM intacte.
3. **Multi-projets** : basculer entre <100 projets rapidement (< 50 ms).
4. **Switch de LLM** : changer de provider (Codex → Gemini → Claude) sans
   perdre le contexte, via opencode.
5. **Orchestration** : un conductor agent qui spawn des sous-agents par ticket,
   avec validation + merge automatique.
6. **Kanban terminal** : backlog + historique + séquencement des US, clickable,
   drag-and-drop, façon Synara mais sans son défaut de RAM.
7. **Git minimal** : commit + graph + checkout d'anciennes versions.

---

## Décisions clés

### Terminal : Kaku (WezTerm fork)

Kaku (par tw93) est un fork optimisé de WezTerm, Rust, GPU-acceleré,
construit pour l'AI coding. Il fournit déjà :

- Panes natifs (moteur mux WezTerm) — pas besoin de tmux/Zellij
- Tabs et workspaces WezTerm (= projets)
- Config Lua (API WezTerm complète)
- lazygit intégré, yazi intégré, AI panel natif
- Persistance via mux domains

**Décision : PAS de tmux, PAS de Zellij.** Les layer par-dessus Kaku
doublerait le multiplexing inutilement et gaspillerait de la RAM.

Alternatives écartées :
- Warp, Tabby : trop lourds (Electron)
- Ghostty : non retenu par préférence
- Rio : bon fallback léger mais Kaku a plus de fonctionnalités natives

### Multiplexing : natif Kaku/WezTerm

Les workspaces WezTerm = un par projet. Le mux garde les processus en vie
au détachement. Au retour : `Cmd+Shift+P` → nom du projet → tout est là.

### Agents : conductor + sous-agents

Au lieu d'une instance opencode par ticket (approche Synara, RAM-explosive),
un seul **conductor** opencode persistant qui spawn des sous-agents à la
demande via le système `task` natif d'opencode.

 | Approche Synara | Approche conductor |
 |----------------|-------------------|
 | 1 opencode/ticket (~1 Go chacun) | 1 conductor + sous-agents à contexte isolé |
 | 10 tickets = ~10 Go RAM | 10 tickets = ~1,5 Go RAM |
 | Pas de validation centrale | Conductor review avant merge |
 | Conflits de branches | Branches git isolées |

### UI : ratatui (Rust)

Pour le Kanban et les panneaux customs : **ratatui**.
- Match l'écosystème Kaku (Rust)
- Support natif souris / clic / drag
- Performant
- Références validantes : lazygit, yazi, bottom

---

## Architecture

```
Kaku (WezTerm fork, ~80 Mo, panes natifs)
┌──────────────────────────────────────────────────────┐
│  WORKSPACE "PKdev"            [Cmd+Shift+P = switch] │
├───────────────┬──────────────────────────────────────┤
│               │  CONDUCTOR (opencode persistant)     │
│  KANBAN TUI   │  - garde le plan + contexte repo     │
│  (ratatui)    │  - spawn les sous-agents par ticket  │
│  clickable    │  - valide les PR avant merge         │
│  drag & drop  │                                      │
│               ├──────────────────────────────────────┤
│  Backlog ─────┤  SOUS-AGENT LOGS (live)              │
│  Doing  ─────┤  US-12 [feat/us-12] implémente...    │
│  Review ─────┤  US-13 [feat/us-13] tests en cours...  │
│  Done   ─────┤  US-14 merged ✓                      │
├───────────────┴──────────────────────────────────────┤
│  GIT PANEL        │  YAZI (fichiers)                 │
│  graph + commit   │  Cmd+Shift+Y                     │
│  + checkout       │                                  │
└──────────────────────────────────────────────────────┘
```

### Panneaux

| Paneau | Rôle | Techno |
|--------|------|--------|
| Conductor | opencode persistant, switch LLM, orchestration | opencode |
| Kanban | backlog / doing / review / done, clickable | Rust + ratatui |
| Sous-agent logs | statut live des workers | Rust + ratatui (ou tmux-like) |
| Git | graph + commit + checkout | Rust + ratatui + lib git2 |
| Fichiers | explorateur | yazi (intégré Kaku) |
| Project card | icône + métadonnées projet | Rust + ratatui |

---

## Flux d'un ticket

```
1. US ajoutée au backlog (clic ou drag dans le Kanban)
   → Kanban écrit le statut dans tickets.db (SQLite)

2. Clic "Start" sur l'US
   → Kanban met statut = "doing" dans tickets.db
   → Conductor lit la DB, spawn un sous-agent pour cette US

3. Sous-agent travaille sur une branche isolée :
   git checkout -b feat/us-12
   → implémente
   → lance les tests
   → ouvre une PR draft
   → écrit son statut dans tickets.db

4. Conductor review la PR :
   ✓ valide → merge → cleanup branche → US passe à "Done"
   ✗ refuse → feedback au sous-agent → itération

5. Kanban se rafraîchit en live (watch de tickets.db)
```

---

## Coordination : tickets.db

Le Kanban TUI et le conductor partagent une source de vérité unique.

```sql
-- tickets.db (SQLite)

tickets (
  id          TEXT PRIMARY KEY,     -- "US-12"
  titre       TEXT,
  statut      TEXT,                 -- backlog|doing|review|done
  branche     TEXT,                 -- "feat/us-12"
  agent_id    TEXT,                 -- référence au sous-agent
  projet      TEXT,                 -- workspace/projet
  créé_le     TIMESTAMP,
  terminé_le  TIMESTAMP
)

agents (
  id              TEXT PRIMARY KEY,
  ticket_id       TEXT,
  statut          TEXT,             -- running|idle|done|error
  tokens_utilisés INTEGER,
  derniere_action TEXT,
  timestamp       TIMESTAMP
)

history (
  id          INTEGER PRIMARY KEY,
  timestamp   TIMESTAMP,
  event       TEXT,                 -- "ticket_started", "merged", ...
  ticket_id   TEXT,
  détail      TEXT
)
```

### Pont MCP (optionnel mais recommandé)

Exposer `tickets.db` comme un serveur MCP. Le conductor peut alors appeler
des outils natifs :
- `get_next_ticket()`
- `mark_done(ticket_id)`
- `get_active_agents()`

opencode parle déjà MCP → intégration native, zéro glue code.

---

## Scope Git (simplifié)

Uniquement :
- **Commit** (avec input de message)
- **Graph** (`git log --oneline --graph`, live)
- **Checkout** d'un commit ancien / créer une branche depuis un commit

Hors scope : stage interactif complexe, rebase, cherry-pick, stash avancé.

Implémentation : mini panneau dans le TUI ratatui via la lib `git2` (~100 lignes).
Fallback temporaire : `tig` (zéro effort, ~5 Mo, fait exactement ces 3 choses).

---

## Stack technique

| Couche | Techno | RAM estimée |
|--------|--------|-------------|
| Terminal | Kaku (WezTerm fork) | ~80 Mo |
| Config + switcher | Lua (API WezTerm) | négligeable |
| Kanban TUI | Rust + ratatui | ~20 Mo |
| Git panel | Rust + ratatui + git2 | inclus |
| Project card TUI | Rust + ratatui | inclus |
| Fichiers | yazi (intégré Kaku) | ~15 Mo |
| Base de coordination | SQLite | ~10 Mo |
| Conductor | opencode | ~1 Go |
| Sous-agents | opencode task (à la demande) | variable |
| **Total base** | | **~1,1 Go** |

---

## Roadmap

### Phase 1 — Socceau (immédiatement utile)
- [ ] Config Kaku : layout par défaut (panes), keybindings
- [ ] Project switcher Lua : fuzzy-liste des <100 projets, switch de workspace
- [ ] tickets.db : schéma SQLite + migrations
- [ ] Kanban TUI read-only : affiche les tickets depuis la DB

### Phase 2 — Kanban interactif
- [ ] Kanban clickable : ajout/édition de tickets
- [ ] Drag & drop entre colonnes
- [ ] Git panel : commit + graph + checkout
- [ ] Project card TUI : icône + métadonnées

### Phase 3 — Orchestration agents
- [ ] Prompts conductor : spawn / validation / merge
- [ ] Pont MCP : tickets.db ↔ conductor
- [ ] Sous-agents + branches git
- [ ] Auto-merge sur validation
- [ ] Logs live des sous-agents

### Phase 4 — Polish
- [ ] Persistance workspace au redémarrage Kaku
- [ ] Récupération de crash (reprise des sous-agents)
- [ ] Stats : tokens utilisés par ticket, temps par US
- [ ] Thèmes / apparence

---

## Fichiers du projet (à créer)

```
DEVpi/
├── PROJECT.md                  # ce document
├── kaku/
│   └── kaku.lua                # config Kaku (panes, switcher, keybindings)
├── kanban/                     # TUI Kanban (Rust + ratatui)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── ui/                 # vues Kanban, git, project card
│       ├── db/                 # couche SQLite
│       └── git/                # panel git (git2)
├── mcp-bridge/                 # serveur MCP tickets.db ↔ conductor
│   └── ...
├── conductor/                  # prompts et config du conductor opencode
│   ├── AGENTS.md
│   └── prompts/
└── projects/                   # registry des <100 projets
    └── projects.json           # { path, icon, metadata }
```

---

## Inspirations et références

- **Kaku** — https://github.com/tw93/Kaku (terminal, WezTerm fork)
- **Synara** — https://github.com/Emanuele-web04/synara (Kanban + agents,
  approche desktop lourde qu'on inverse)
- **opencode** — conductor + système de sous-agents natif (`task`)
- **ratatui** — framework TUI Rust (lazygit, yazi, bottom comme références)
