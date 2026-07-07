# PKdev — Gestionnaire d'Agents

Environnement de dev léger, 100% terminal, pour orchestrer des agents IA.
Construit autour de **Kaku** (WezTerm fork) + un **Kanban TUI** clickable en
Rust + **opencode** comme conductor, le tout coordonné par **herdr**.

## Démarrage rapide

### 1. Installer les fonts

```bash
chmod +x scripts/install-fonts.sh
./scripts/install-fonts.sh
```

Installe : JetBrainsMono Nerd Font, FiraCode Nerd Font, Hack Nerd Font,
Inter, Poppins.

### 2. Compiler le Kanban

```bash
cd kanban && cargo build --release
```

Le binaire se trouve dans `kanban/target/release/kanban`.

### 3. Lancer

**Option A — dans Kaku (recommandé)** : le workspace complet (terminal +
kanban + intégration herdr).

```bash
./start
```

`start` compile le kanban si besoin, ferme les instances précédentes et lance
Kaku avec la config du projet (`kaku/workspace.lua`), cwd = dossier du projet.

**Option B — kanban seul, dans n'importe quel terminal** :

```bash
cd /Users/clm/Documents/GitHub/PROJECTS/PKdev
./kanban/target/release/kanban
```

Au premier lancement, `tickets.db` est créée automatiquement (schéma complet +
données de démo). Le kanban doit être lancé **depuis le dossier du projet**
pour trouver la DB et résoudre les chemins des projets.

## Vue d'ensemble

L'app démarre sur l'écran **Projets**. Chaque projet pointe vers un dossier
(cwd) ; en l'ouvrant on entre dans le **board** (4 colonnes : backlog / doing /
review / done). Chaque ticket ouvre une **vue détail** d'où partent les
branches git et les agents opencode.

### Projets

| Touche | Action |
|--------|--------|
| `Entrée` / clic | Ouvrir le projet sélectionné |
| `a` | Ajouter un projet — **navigateur de dossiers TUI** |
| `d` | Supprimer (confirmation `y/n`) |
| `Tab` | Basculer vue grille ⇄ vue liste |
| `← →` / `h l` | Naviguer entre projets |
| `j/k` | Naviguer (vue liste) |
| `t` | Changer de thème |
| `?` | Aide |

#### Navigateur de dossiers (TUI)

S'ouvre sur `a`. 100% terminal, aucune dépendance GUI.

| Touche | Action |
|--------|--------|
| `j/k` / `↑ ↓` | Naviguer |
| `Entrée` / `→` | Ouvrir un dossier, ou valider « ✓ Utiliser ce dossier » |
| `h` / `←` / `Backspace` | Dossier parent |
| `n` | **Créer un nouveau dossier** (saisie du nom, `Entrée` = créer) |
| `Échap` | Annuler |

> **Auto-réparation** : si le chemin d'un projet n'existe plus (dossier
> renommé/déplacé), le kanban le recale automatiquement sur le cwd courant
> tant que c'est un dépôt git. Plus besoin de supprimer/recréer.

### Board (tickets)

| Touche | Action |
|--------|--------|
| `h j k l` / flèches | Naviguer entre colonnes et tickets |
| `H` / `L` | Déplacer le ticket de colonne (gauche / droite) |
| `a` | Ajouter un ticket |
| `d` | Supprimer le ticket |
| `Entrée` / clic | Ouvrir la vue détail |
| clic droit | Menu contextuel (détail / déplacer / supprimer) |
| glisser-déposer | Déplacer entre colonnes |
| `p` / `Échap` | Retour aux projets |
| `r` | Recharger depuis la DB |
| `t` / `?` | Thème / Aide |

### Vue détail d'un ticket

| Touche | Action |
|--------|--------|
| saisie + `Entrée` | Envoyer un prompt au conductor |
| `Ctrl+F` | Focus sur l'agent conductor |
| `Ctrl+U` | Vider la saisie |
| `Échap` | Retour au board |

## Intégration herdr

Le kanban pilote herdr (`HERDR_BIN_PATH` ou `herdr` dans le `PATH`) pour :

- lister les agents (`herdr agent list --json`) — indicateur de statut sur
  chaque carte (● working / blocked / idle),
- démarrer un agent par ticket,
- focus + envoi de prompt au conductor.

herdr expose `HERDR_WORKSPACE_ID`, `HERDR_TAB_ID`, `HERDR_PANE_ID` dans chaque
pane géré : le kanban s'en sert pour cibler le bon workspace à l'ouverture
d'agents.

## Raccourcis Kaku

| Raccourci | Action |
|-----------|--------|
| `Cmd+Shift+T` | Basculer Dracula / Catppuccin |
| `Cmd+D` | Split horizontal |
| `Cmd+Shift+D` | Split vertical |
| `Cmd+Opt+←↑↓→` | Naviguer entre panes |
| `Cmd+W` | Fermer le pane |
| `Cmd+T` | Nouvel onglet |

## Structure du projet

```
PKdev/
├── PROJECT.md              Spec complète (architecture, roadmap)
├── README.md               Ce fichier
├── README_en.md            English version
├── start                   Lanceur Kaku (compile + workspace)
├── kanban/                 Kanban TUI (Rust + ratatui)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         Entrée + event loop + init DB
│       ├── app.rs          État + logique + événements + navigateur
│       ├── db.rs           Couche SQLite (CRUD)
│       ├── herdr.rs        Pont vers herdr (agents, focus)
│       ├── ui.rs           Rendu ratatui
│       └── theme.rs        Palettes Dracula + Catppuccin
├── kaku/
│   ├── workspace.lua       Config Kaku (fonts, thème, raccourcis)
│   └── kaku.lua            Référence layout multi-panes
├── scripts/
│   ├── install-fonts.sh    Nerd Fonts + Inter + Poppins
│   └── schema.sql          Schéma SQLite + données de démo
└── tickets.db              Créée au premier lancement (gitignored)
```

## Thèmes

Six palettes intégrées, switchables avec `t` :

**Thèmes sombres**
- **Catppuccin Mocha** (par défaut) — doux, pastel
- **Dracula** — contrasté, néon

**Thèmes clairs**
- **Sakura** — blanc rosé doux, fleurs de cerisier
- **Nord Light** — gris-bleu frais, scandinave
- **Solarized Light** — crème chaud, contrastes équilibrés
- **Gruvbox Light** — rétro terreux, ambre/crème

## Ce qui marche

- Écran Projets (vue grille + vue liste, `Tab`)
- Navigateur de dossiers TUI avec création de dossiers
- Auto-réparation des chemins de projets (déplacement/renommage)
- Board 4 colonnes, navigation clavier + souris, drag & drop
- Vue détail, prompts, branches git par ticket
- Démarrage d'agent opencode (workspace herdr courant)
- Persistance SQLite
- Thèmes Dracula, Catppuccin + 4 thèmes clairs (Sakura, Nord, Solarized, Gruvbox)
- Support herdr imbriqué (pour voir les agents travailler)

## Roadmap

Voir `PROJECT.md` pour le détail complet.

- Pont MCP (tickets.db <-> conductor opencode)
- Sous-agents conductor (spawn/validate)
- Auto-merge sur validation
- Logs live des workers
