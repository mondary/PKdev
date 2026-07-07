use rusqlite::{params, Connection};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct Ticket {
    pub id: String,
    pub projet_id: String,
    pub titre: String,
    pub description: String,
    pub statut: String,
    pub priorite: i32,
    pub branche: String,
    pub worktree: String,
    pub agent_id: String,
}

#[derive(Clone, Debug)]
pub struct Prompt {
    pub texte: String,
    pub timestamp: String,
}

#[derive(Clone, Debug)]
pub struct Project {
    pub id: String,
    pub nom: String,
    pub chemin: String,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode = WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let db = Database { conn };
        db.ensure_prompts_table()?;
        Ok(db)
    }

    pub fn execute_batch(&self, sql: &str) -> rusqlite::Result<()> {
        self.conn.execute_batch(sql)
    }

    fn ensure_prompts_table(&self) -> rusqlite::Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS prompts (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                ticket_id   TEXT NOT NULL,
                texte       TEXT NOT NULL,
                timestamp   TEXT DEFAULT (datetime('now'))
            );",
        )?;
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS projects (
                id          TEXT PRIMARY KEY,
                nom         TEXT NOT NULL,
                chemin      TEXT NOT NULL,
                icone       TEXT DEFAULT '',
                cree_le     TEXT DEFAULT (datetime('now'))
            );",
        )?;
        Ok(())
    }

    pub fn liste_projets(&self) -> rusqlite::Result<Vec<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, nom, chemin FROM projects ORDER BY cree_le ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                nom: row.get(1)?,
                chemin: row.get(2)?,
            })
        })?;
        rows.collect()
    }

    pub fn nb_tickets_projet(&self, projet_id: &str) -> i64 {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM tickets WHERE projet_id = ?1",
                params![projet_id],
                |row| row.get(0),
            )
            .unwrap_or(0)
    }

    pub fn ajouter_projet(&self, id: &str, nom: &str, chemin: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO projects (id, nom, chemin) VALUES (?1, ?2, ?3)",
            params![id, nom, chemin],
        )?;
        Ok(())
    }

    pub fn update_chemin(&self, id: &str, chemin: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE projects SET chemin = ?1 WHERE id = ?2",
            params![chemin, id],
        )?;
        Ok(())
    }

    pub fn supprimer_projet(&self, id: &str) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM tickets WHERE projet_id = ?1", params![id])?;
        self.conn.execute("DELETE FROM prompts WHERE ticket_id IN (SELECT id FROM tickets WHERE projet_id = ?1)", params![id])?;
        self.conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn tickets_par_projet(&self, projet_id: &str) -> rusqlite::Result<Vec<Ticket>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, projet_id, titre, description, statut, priorite, branche, worktree, agent_id
             FROM tickets WHERE projet_id = ?1
             ORDER BY priorite DESC, cree_le ASC",
        )?;
        let rows = stmt.query_map(params![projet_id], |row| {
            Ok(Ticket {
                id: row.get(0)?,
                projet_id: row.get(1)?,
                titre: row.get(2)?,
                description: row.get(3)?,
                statut: row.get(4)?,
                priorite: row.get(5)?,
                branche: row.get(6)?,
                worktree: row.get(7)?,
                agent_id: row.get(8)?,
            })
        })?;
        rows.collect()
    }

    pub fn tous_les_tickets(&self) -> rusqlite::Result<Vec<Ticket>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, projet_id, titre, description, statut, priorite, branche, worktree, agent_id
             FROM tickets ORDER BY projet_id, priorite DESC, cree_le ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Ticket {
                id: row.get(0)?,
                projet_id: row.get(1)?,
                titre: row.get(2)?,
                description: row.get(3)?,
                statut: row.get(4)?,
                priorite: row.get(5)?,
                branche: row.get(6)?,
                worktree: row.get(7)?,
                agent_id: row.get(8)?,
            })
        })?;
        rows.collect()
    }

    pub fn ajouter_ticket(
        &self,
        id: &str,
        projet_id: &str,
        titre: &str,
        statut: &str,
    ) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO tickets (id, projet_id, titre, statut) VALUES (?1, ?2, ?3, ?4)",
            params![id, projet_id, titre, statut],
        )?;
        self.log_history("ticket_created", Some(id), "Ticket créé")
    }

    pub fn deplacer_ticket(&self, id: &str, nouveau_statut: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE tickets SET statut = ?1 WHERE id = ?2",
            params![nouveau_statut, id],
        )?;
        if nouveau_statut == "done" {
            self.conn.execute(
                "UPDATE tickets SET termine_le = datetime('now') WHERE id = ?1",
                params![id],
            )?;
        }
        self.log_history("ticket_moved", Some(id), &format!("-> {}", nouveau_statut))
    }

    pub fn supprimer_ticket(&self, id: &str) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM tickets WHERE id = ?1", params![id])?;
        self.log_history("ticket_deleted", Some(id), "Ticket supprimé")
    }

    pub fn prochain_id(&self, projet_id: &str) -> rusqlite::Result<String> {
        let prefix = projet_id.to_uppercase();
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM tickets WHERE projet_id = ?1",
            params![projet_id],
            |row| row.get(0),
        )?;
        Ok(format!("{}-{:02}", prefix, count + 1))
    }

    pub fn update_worktree(
        &self,
        id: &str,
        worktree: &str,
        branche: &str,
    ) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE tickets SET worktree = ?1, branche = ?2 WHERE id = ?3",
            params![worktree, branche, id],
        )?;
        Ok(())
    }

    pub fn ajouter_prompt(&self, ticket_id: &str, texte: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO prompts (ticket_id, texte) VALUES (?1, ?2)",
            params![ticket_id, texte],
        )?;
        Ok(())
    }

    pub fn prompts_pour(&self, ticket_id: &str) -> rusqlite::Result<Vec<Prompt>> {
        let mut stmt = self.conn.prepare(
            "SELECT texte, timestamp FROM prompts WHERE ticket_id = ?1 ORDER BY id ASC",
        )?;
        let rows = stmt.query_map(params![ticket_id], |row| {
            Ok(Prompt {
                texte: row.get(0)?,
                timestamp: row.get(1)?,
            })
        })?;
        rows.collect()
    }

    fn log_history(
        &self,
        event: &str,
        ticket_id: Option<&str>,
        detail: &str,
    ) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO history (event, ticket_id, detail) VALUES (?1, ?2, ?3)",
            params![event, ticket_id, detail],
        )?;
        Ok(())
    }
}
