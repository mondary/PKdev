# DEVpi — Gestionnaire d'Agents

Environnement de dev léger, 100% terminal, pour orchestrer des agents IA.
Construit autour de **Kaku** (WezTerm fork) + un **Kanban TUI** clickable en
Rust + **opencode** comme conductor.

## Démarrage rapide

### 1. Installer les fonts

```bash
chmod +x scripts/install-fonts.sh
./scripts/install-fonts.sh
```

Installe : JetBrainsMono Nerd Font, FiraCode Nerd Font, Hack Nerd Font,
Inter, Poppins. (Futura = font commercial, voir le script pour les alternatives.)

### 2. Compiler le Kanban

```bash
cd kanban
cargo build --release
```

Le binaire se trouve dans `kanban/target/release/kanban`.

### 3. Lancer le Kanban

```bash
./kanban/target/debug/kanban
# ou en release :
./kanban/target/release/kanban
```

Au premier lancement, `tickets.db` est créée automatiquement avec le schéma
complet + 12 tickets de démo.

Vous pouvez aussi spécifier un chemin de DB et un ID de projet :

```bash
./kanban/target/release/kanban /chemin/vers/tickets.db mon-projet
```

### 4. (Optionnel) Référence Kaku

Le fichier `kaku/kaku.lua` est une **référence** montrant comment configurer
un layout multi-panes + project switcher. À adapter selon vos besoins, ou à
ignorer — le Kanban fonctionne dans n'importe quel terminal.

## Commandes du Kanban

### Navigation

| Touche | Action |
|--------|--------|
| `h` `j` `k` `l` ou `←` `↓` `↑` `→` | Naviguer entre colonnes et tickets |
| `H` | Déplacer le ticket vers la colonne de gauche |
| `L` | Déplacer le ticket vers la colonne de droite |
| Clic souris | Sélectionner un ticket |
| Scroll | Naviguer dans la colonne |

### Actions

| Touche | Action |
|--------|--------|
| `a` | Ajouter un ticket (mode insertion) |
| `Entrée` | Valider l'ajout |
| `Échap` | Annuler l'ajout / fermer l'aide |
| `d` | Supprimer le ticket sélectionné |
| `t` | Basculer entre Dracula et Catppuccin |
| `r` | Recharger depuis la DB |
| `?` | Afficher l'aide |
| `q` | Quitter |

## Commandes Kaku

| Raccourci | Action |
|-----------|--------|
| `Cmd+Shift+P` | Switcher de projet (fuzzy) |
| `Cmd+Shift+T` | Basculer Dracula / Catppuccin |
| `Cmd+D` | Split horizontal |
| `Cmd+Shift+D` | Split vertical |
| `Cmd+Opt+←↑↓→` | Naviguer entre panes |

## Structure du projet

```
DEVpi/
├── PROJECT.md              Spec complète (architecture, roadmap)
├── README.md               Ce fichier
├── kanban/                 Kanban TUI (Rust + ratatui)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         Entrée + event loop + init DB
│       ├── app.rs          État + logique + événements
│       ├── db.rs           Couche SQLite (CRUD)
│       ├── ui.rs           Rendu ratatui (colonnes, cartes, aide)
│       └── theme.rs        Palettes Dracula + Catppuccin
├── kaku/
│   └── kaku.lua            Config Kaku (fonts, thème, panes, switcher)
├── conductor/              Prompts du conductor (à venir)
│   └── prompts/
├── scripts/
│   ├── install-fonts.sh    Installation des Nerd Fonts + Inter + Poppins
│   └── schema.sql          Schéma SQLite + données de démo
└── tickets.db              Créée automatiquement au premier lancement
```

## Thèmes

Deux palettes intégrées, switchables avec `t` :

- **Catppuccin Mocha** (par défaut) — doux, pastel
- **Dracula** — contrasté, néon

Les deux utilisent les couleurs officielles de chaque thème.

## Ce qui marche maintenant

- Kanban TUI avec 4 colonnes (Backlog / En cours / Review / Terminé)
- Navigation clavier complète (hjkl + flèches)
- Support souris (clic pour sélectionner, scroll)
- Ajout / suppression / déplacement de tickets
- Persistance SQLite (tickets.db)
- Auto-init de la DB au premier lancement
- Thèmes Dracula + Catppuccin switchables
- Écran d'aide (?)
- Config Kaku avec panes + project switcher

## Prochaines étapes (roadmap)

Voir `PROJECT.md` pour le détail complet.

- Git panel (commit + graph + checkout)
- Pont MCP (tickets.db <-> conductor opencode)
- Sous-agents + worktrees git par ticket
- Auto-merge sur validation
- Logs live des workers
