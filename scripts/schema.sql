-- ============================================================
--  tickets.db — schéma + données de démo
--  Usage : sqlite3 tickets.db < schema.sql
-- ============================================================

PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

DROP TABLE IF EXISTS history;
DROP TABLE IF EXISTS agents;
DROP TABLE IF EXISTS tickets;
DROP TABLE IF EXISTS projects;

-- ------------------------------------------------------------
-- Projets (workspaces)
-- ------------------------------------------------------------
CREATE TABLE projects (
  id          TEXT PRIMARY KEY,
  nom         TEXT NOT NULL,
  chemin      TEXT NOT NULL,
  icone       TEXT DEFAULT '',
  cree_le     TEXT DEFAULT (datetime('now'))
);

-- ------------------------------------------------------------
-- Tickets / User Stories
-- ------------------------------------------------------------
CREATE TABLE tickets (
  id          TEXT PRIMARY KEY,
  projet_id   TEXT NOT NULL,
  titre       TEXT NOT NULL,
  description TEXT DEFAULT '',
  statut      TEXT NOT NULL DEFAULT 'backlog'
              CHECK (statut IN ('backlog', 'doing', 'review', 'done')),
  priorite    INTEGER NOT NULL DEFAULT 0,
  branche     TEXT DEFAULT '',
  worktree    TEXT DEFAULT '',
  agent_id    TEXT DEFAULT '',
  cree_le     TEXT DEFAULT (datetime('now')),
  termine_le  TEXT,
  FOREIGN KEY (projet_id) REFERENCES projects(id)
);

CREATE INDEX idx_tickets_statut   ON tickets (statut);
CREATE INDEX idx_tickets_projet   ON tickets (projet_id);

-- ------------------------------------------------------------
-- Agents (sous-agents lancés par le conductor)
-- ------------------------------------------------------------
CREATE TABLE agents (
  id               TEXT PRIMARY KEY,
  ticket_id        TEXT,
  statut           TEXT NOT NULL DEFAULT 'idle'
                   CHECK (statut IN ('idle', 'running', 'done', 'error')),
  tokens_utilises  INTEGER DEFAULT 0,
  derniere_action  TEXT DEFAULT '',
  mis_a_jour_le    TEXT DEFAULT (datetime('now')),
  FOREIGN KEY (ticket_id) REFERENCES tickets(id)
);

-- ------------------------------------------------------------
-- Historique (audit trail de tous les événements)
-- ------------------------------------------------------------
CREATE TABLE history (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp   TEXT DEFAULT (datetime('now')),
  event       TEXT NOT NULL,
  ticket_id   TEXT,
  detail      TEXT DEFAULT ''
);

-- ============================================================
--  Données de démo
-- ============================================================

INSERT INTO projects (id, nom, chemin, icone) VALUES
  ('devpi', 'DEVpi — Gestionnaire d''Agents', '/Users/clm/Documents/GitHub/TESTS/DEVpi', '');

INSERT INTO tickets (id, projet_id, titre, description, statut, priorite) VALUES
  ('US-01', 'devpi', 'Config Kaku : layout panes + switcher', 'Mettre en place kaku.lua avec le layout 4 panes et le project switcher fuzzy', 'done',   3),
  ('US-02', 'devpi', 'Schéma tickets.db SQLite',              'Créer le schéma de coordination conductor/kanban', 'done',   3),
  ('US-03', 'devpi', 'Kanban TUI read-only',                  'Afficher les tickets depuis la DB en 4 colonnes',  'done',   2),
  ('US-04', 'devpi', 'Kanban clickable + drag & drop',        'Rendre le Kanban interactif : ajout, déplacement, édition', 'doing',  2),
  ('US-05', 'devpi', 'Support souris (clic sélection)',       'Cliquer sur un ticket pour le sélectionner', 'doing',  1),
  ('US-06', 'devpi', 'Git panel : commit + graph + checkout', 'Mini panneau git via lib git2 dans le TUI', 'review', 2),
  ('US-07', 'devpi', 'Thèmes Dracula + Catppuccin',           'Palettes switchables dans le Kanban', 'review', 1),
  ('US-08', 'devpi', 'Pont MCP tickets.db <-> conductor',     'Exposer la DB comme serveur MCP pour opencode', 'backlog', 2),
  ('US-09', 'devpi', 'Prompts conductor : spawn/validate',    'Définir les prompts du conductor opencode', 'backlog', 2),
  ('US-10', 'devpi', 'Sous-agents + worktrees git',           'Spawn un sub-agent par ticket dans un worktree isolé', 'backlog', 3),
  ('US-11', 'devpi', 'Auto-merge sur validation',             'Conductor merge automatiquement après review OK', 'backlog', 1),
  ('US-12', 'devpi', 'Logs live des sous-agents',             'Panneau affichant le statut temps réel des workers', 'backlog', 1);

INSERT INTO history (event, ticket_id, detail) VALUES
  ('ticket_created', 'US-01', 'Config Kaku initiale'),
  ('ticket_moved',   'US-01', 'backlog -> doing'),
  ('ticket_moved',   'US-01', 'doing -> done'),
  ('ticket_created', 'US-04', 'Kanban interactif démarré'),
  ('ticket_moved',   'US-06', 'doing -> review');
