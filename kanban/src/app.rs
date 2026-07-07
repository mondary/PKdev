use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use crate::db::{Database, Project, Ticket};
use crate::herdr;
use crate::theme::Theme;

pub const COLONNES: [&str; 4] = ["backlog", "doing", "review", "done"];

#[derive(PartialEq, Clone, Copy)]
pub enum Mode {
    Projects,
    ProjectNew,
    ProjectDelete,
    Normal,
    Insert,
    Detail,
    ContextMenu,
}

#[derive(Clone)]
struct HitZone {
    col: usize,
    ticket: usize,
    rect: Rect,
}

#[derive(Clone)]
struct ProjectZone {
    idx: usize,
    rect: Rect,
}

struct DragState {
    ticket_id: String,
    col_source: usize,
}

pub fn choisir_dossier() -> Option<String> {
    let output = std::process::Command::new("osascript")
        .args([
            "-e",
            r#"tell application "System Events" to get POSIX path of (choose folder with prompt "Sélectionner le dossier du projet")"#,
        ])
        .output()
        .ok()?;
    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() { Some(path) } else { None }
    } else {
        None
    }
}

pub struct App {
    pub tickets: Vec<Ticket>,
    pub all_tickets: Vec<Ticket>,
    pub prompts: Vec<crate::db::Prompt>,
    pub agents: Vec<herdr::HerdrAgent>,
    pub projects: Vec<Project>,
    pub project_cursor: usize,
    pub project_view_list: bool,
    pub col_cursor: usize,
    pub ticket_cursor: usize,
    pub mode: Mode,
    pub input: String,
    pub prompt_input: String,
    pub theme: Theme,
    pub project_id: String,
    pub project_dir: String,
    pub should_quit: bool,
    pub message: String,
    pub show_help: bool,
    pub hit_zones: Vec<HitZone>,
    pub project_zones: Vec<ProjectZone>,
    pub drag: Option<DragState>,
    pub show_context_menu: bool,
    pub context_cursor: usize,
    db: Database,
}

impl App {
    pub fn new(db: Database) -> Self {
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let mut app = App {
            tickets: Vec::new(),
            all_tickets: Vec::new(),
            prompts: Vec::new(),
            agents: Vec::new(),
            projects: Vec::new(),
            project_cursor: 0,
            project_view_list: false,
            col_cursor: 0,
            ticket_cursor: 0,
            mode: Mode::Projects,
            input: String::new(),
            prompt_input: String::new(),
            theme: Theme::catppuccin(),
            project_id: String::new(),
            project_dir: cwd,
            should_quit: false,
            message: String::new(),
            show_help: false,
            hit_zones: Vec::new(),
            project_zones: Vec::new(),
            drag: None,
            show_context_menu: false,
            context_cursor: 0,
            db,
        };

        app.load_projects();
        app.sync_agents();
        app
    }

    // ----------------------------------------------------------
    //  Projets
    // ----------------------------------------------------------
    pub fn load_projects(&mut self) {
        self.projects = self.db.liste_projets().unwrap_or_default();
        self.all_tickets = self.db.tous_les_tickets().unwrap_or_default();

        if self.projects.is_empty() {
            let nom = std::path::Path::new(&self.project_dir)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "projet".into());
            let id = nom.to_lowercase().replace(' ', "-");
            let _ = self.db.ajouter_projet(&id, &nom, &self.project_dir);
            self.projects = self.db.liste_projets().unwrap_or_default();
            self.all_tickets = self.db.tous_les_tickets().unwrap_or_default();
        }
    }

    pub fn nb_tickets_projet(&self, projet_id: &str) -> i64 {
        self.db.nb_tickets_projet(projet_id)
    }

    pub fn selectionner_projet(&mut self) {
        let p = self.projects.get(self.project_cursor).cloned();
        if let Some(p) = p {
            self.project_id = p.id.clone();
            self.project_dir = p.chemin.clone();
            self.reload();
            self.sync_agents();
            self.mode = Mode::Normal;
            self.col_cursor = 0;
            self.ticket_cursor = 0;
            self.message = format!("{} — {} tickets", p.id, self.tickets.len());
            self.lancer_conductor();
        }
    }

    pub fn creer_projet_depuis_input(&mut self) {
        let chemin = self.input.trim().trim_end_matches('/').to_string();
        if chemin.is_empty() {
            self.mode = Mode::Projects;
            return;
        }
        let nom = std::path::Path::new(&chemin)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "projet".into());
        let id = nom.to_lowercase().replace(' ', "-");

        let _ = self.db.ajouter_projet(&id, &nom, &chemin);
        self.load_projects();
        self.project_cursor = self.projects.iter().position(|p| p.id == id).unwrap_or(0);
        self.mode = Mode::Projects;
        self.input.clear();
        self.message = format!("Projet {} ajouté", id);
    }

    pub fn retour_projets(&mut self) {
        self.mode = Mode::Projects;
        self.load_projects();
        self.message.clear();
    }

    pub fn basculer_vue_projets(&mut self) {
        self.project_view_list = !self.project_view_list;
        self.message = if self.project_view_list {
            "Vue liste".into()
        } else {
            "Vue tableau".into()
        };
    }

    pub fn confirmer_suppression_projet(&mut self) {
        if self.projects.is_empty() {
            return;
        }
        self.mode = Mode::ProjectDelete;
    }

    pub fn supprimer_projet_confirme(&mut self) {
        let p = self.projects.get(self.project_cursor).cloned();
        if let Some(p) = p {
            let nb = self.nb_tickets_projet(&p.id);
            let _ = self.db.supprimer_projet(&p.id);
            self.load_projects();
            if self.project_cursor >= self.projects.len() && !self.projects.is_empty() {
                self.project_cursor = self.projects.len() - 1;
            }
            self.message = format!("{} supprimé ({} tickets)", p.id, nb);
        }
        self.mode = Mode::Projects;
    }

    pub fn annuler_suppression(&mut self) {
        self.mode = Mode::Projects;
        self.message.clear();
    }

    // ----------------------------------------------------------
    //  Tickets
    // ----------------------------------------------------------
    pub fn reload(&mut self) {
        if let Ok(t) = self.db.tickets_par_projet(&self.project_id) {
            self.tickets = t;
        }
        self.clamp_cursor();
    }

    pub fn sync_agents(&mut self) {
        self.agents = herdr::list_agents();
    }

    pub fn agent_pour(&self, ticket_id: &str) -> Option<&herdr::HerdrAgent> {
        self.agents.iter().find(|a| a.name == ticket_id)
    }

    fn clamp_cursor(&mut self) {
        let c = self.colonne_courante().len();
        if self.ticket_cursor >= c && c > 0 {
            self.ticket_cursor = c - 1;
        }
        if c == 0 {
            self.ticket_cursor = 0;
        }
    }

    pub fn colonne(&self, statut: &str) -> Vec<&Ticket> {
        self.tickets.iter().filter(|t| t.statut == statut).collect()
    }
    pub fn colonne_courante(&self) -> Vec<&Ticket> {
        self.colonne(COLONNES[self.col_cursor])
    }
    fn ticket_selectionne(&self) -> Option<&Ticket> {
        self.colonne_courante().get(self.ticket_cursor).copied()
    }
    pub fn nb_colonne(&self, statut: &str) -> usize {
        self.colonne(statut).len()
    }

    // ----------------------------------------------------------
    //  Navigation board
    // ----------------------------------------------------------
    pub fn curseur_gauche(&mut self) {
        if self.col_cursor > 0 { self.col_cursor -= 1; self.ticket_cursor = 0; }
    }
    pub fn curseur_droite(&mut self) {
        if self.col_cursor < COLONNES.len() - 1 { self.col_cursor += 1; self.ticket_cursor = 0; }
    }
    pub fn curseur_haut(&mut self) {
        if self.ticket_cursor > 0 { self.ticket_cursor -= 1; }
    }
    pub fn curseur_bas(&mut self) {
        let c = self.colonne_courante().len();
        if self.ticket_cursor < c.saturating_sub(1) { self.ticket_cursor += 1; }
    }

    pub fn deplacer_gauche(&mut self) {
        if self.col_cursor > 0 {
            if let Some(t) = self.ticket_selectionne() {
                let id = t.id.clone();
                let nv = COLONNES[self.col_cursor - 1];
                let _ = self.db.deplacer_ticket(&id, nv);
                self.col_cursor -= 1;
                self.reload();
            }
        }
    }
    pub fn deplacer_droite(&mut self) {
        if self.col_cursor < COLONNES.len() - 1 {
            if let Some(t) = self.ticket_selectionne() {
                let id = t.id.clone();
                let nv = COLONNES[self.col_cursor + 1];
                let _ = self.db.deplacer_ticket(&id, nv);
                self.col_cursor += 1;
                self.reload();
            }
        }
    }
    pub fn supprimer(&mut self) {
        if let Some(t) = self.ticket_selectionne() {
            let id = t.id.clone();
            let _ = self.db.supprimer_ticket(&id);
            self.reload();
            self.message = format!("{} supprimé", id);
        }
    }

    // ----------------------------------------------------------
    //  Ajout ticket
    // ----------------------------------------------------------
    pub fn entrer_insertion(&mut self) {
        self.mode = Mode::Insert;
        self.input.clear();
    }
    pub fn confirmer_ajout(&mut self) {
        let titre = self.input.trim().to_string();
        if titre.is_empty() { self.mode = Mode::Normal; return; }
        let statut = COLONNES[self.col_cursor];
        if let Ok(id) = self.db.prochain_id(&self.project_id) {
            let _ = self.db.ajouter_ticket(&id, &self.project_id, &titre, statut);
            self.reload();
            self.message = format!("{} créé", id);
        }
        self.mode = Mode::Normal;
        self.input.clear();
    }

    // ----------------------------------------------------------
    //  Vue détaillée
    // ----------------------------------------------------------
    pub fn entrer_detail(&mut self) {
        if self.ticket_selectionne().is_some() {
            let id = self.ticket_selectionne().unwrap().id.clone();
            self.prompts = self.db.prompts_pour(&id).unwrap_or_default();
            self.mode = Mode::Detail;
            self.prompt_input.clear();
        }
    }

    pub fn ticket_detail_id(&self) -> Option<String> {
        if self.mode == Mode::Detail {
            self.colonne_courante().get(self.ticket_cursor).map(|t| t.id.clone())
        } else { None }
    }
    pub fn envoyer_prompt(&mut self) {
        let texte = self.prompt_input.trim().to_string();
        if texte.is_empty() {
            self.mode = Mode::Detail;
            return;
        }
        if let Some(id) = self.ticket_detail_id() {
            let _ = self.db.ajouter_prompt(&id, &texte);
            self.prompts = self.db.prompts_pour(&id).unwrap_or_default();

            let wt = self
                .tickets
                .iter()
                .find(|t| t.id == id)
                .map(|t| t.worktree.clone())
                .unwrap_or_default();

            if wt.is_empty() {
                self.demarrer_worktree(&id);
            }

            let prompt_complet = if let Some(wt) = self.tickets.iter().find(|t| t.id == id).map(|t| &t.worktree) {
                if wt.is_empty() {
                    texte.clone()
                } else {
                    format!("Travaille sur {} dans le worktree {}: {}", id, wt, texte)
                }
            } else {
                texte.clone()
            };

            herdr::send_prompt(&self.project_id, &prompt_complet);
            self.message = format!("Prompt envoyé au conductor pour {}", id);
        }
        self.prompt_input.clear();
    }

    pub fn focus_selection(&mut self) {
        let conductor = self.project_id.clone();
        if herdr::herdr_dispo() {
            herdr::focus_agent(&conductor);
            self.message = format!("Focus -> conductor {}", conductor);
        } else {
            self.message = "Herdr requis pour le focus — lance `herdr` d'abord".into();
        }
    }
    fn lancer_conductor(&mut self) {
        if !herdr::herdr_dispo() {
            self.message = format!(
                "{} — lance opencode dans un terminal, ou démarre Herdr",
                self.project_id
            );
            return;
        }

        let conductor = self.project_id.clone();
        let cwd = self.project_dir.clone();

        if self.agents.iter().any(|a| a.name == conductor) {
            self.message = format!("Conductor {} déjà actif", conductor);
            return;
        }

        if herdr::start_agent(&conductor, &cwd) {
            self.sync_agents();
            self.message = format!("Conductor {} lancé via Herdr", conductor);
        } else {
            self.message = "Herdr détecté mais lancement échoué".into();
        }
    }

    fn demarrer_worktree(&mut self, ticket_id: &str) {
        let branche = format!("feat/{}", ticket_id.to_lowercase());
        let wt_name = format!("{}-{}", self.project_id, ticket_id.to_lowercase());
        let wt_path = format!(
            "{}/{}",
            self.project_dir.trim_end_matches('/'),
            wt_name
        );

        let result = std::process::Command::new("git")
            .args(["worktree", "add", &wt_path, "-b", &branche])
            .current_dir(&self.project_dir)
            .output();

        match result {
            Ok(o) if o.status.success() => {
                let _ = self.db.update_worktree(ticket_id, &wt_path, &branche);
                let _ = self.db.deplacer_ticket(ticket_id, "doing");
                self.reload();
                self.message = format!("{} worktree: {}", ticket_id, wt_path);
            }
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr);
                let first = err.lines().next().unwrap_or("erreur inconnue");
                self.message = format!("{}: {}", ticket_id, first);
            }
            Err(e) => {
                self.message = format!("{} git error: {}", ticket_id, e);
            }
        }
    }

    pub fn basculer_theme(&mut self) {
        let ancien = self.theme.name;
        self.theme = self.theme.next();
        self.message = format!("{} -> {}", ancien, self.theme.name);
    }

    // ----------------------------------------------------------
    //  Événements
    // ----------------------------------------------------------
    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Projects => self.key_projects(key),
            Mode::ProjectNew => self.key_project_new(key),
            Mode::ProjectDelete => self.key_project_delete(key),
            Mode::Normal => self.key_normal(key),
            Mode::Insert => self.key_insert(key),
            Mode::Detail => self.key_detail(key),
            Mode::ContextMenu => self.key_context_menu(key),
        }
    }

    fn key_context_menu(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.show_context_menu = false;
                self.mode = Mode::Normal;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.context_cursor > 0 { self.context_cursor -= 1; }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.context_cursor < 3 { self.context_cursor += 1; }
            }
            KeyCode::Enter => {
                let cursor = self.context_cursor;
                self.show_context_menu = false;
                self.mode = Mode::Normal;
                match cursor {
                    0 => self.entrer_detail(),
                    1 => self.deplacer_gauche(),
                    2 => self.deplacer_droite(),
                    3 => self.supprimer(),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn key_projects(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('?') => self.show_help = !self.show_help,
            KeyCode::Char('t') => self.basculer_theme(),
            KeyCode::Tab => self.basculer_vue_projets(),
            KeyCode::Char('a') => { self.mode = Mode::ProjectNew; self.input.clear(); }
            KeyCode::Char('d') => self.confirmer_suppression_projet(),
            KeyCode::Left | KeyCode::Char('h') => {
                if self.project_cursor > 0 { self.project_cursor -= 1; }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.project_cursor < self.projects.len().saturating_sub(1) { self.project_cursor += 1; }
            }
            KeyCode::Enter => self.selectionner_projet(),
            _ => {}
        }
    }

    fn key_project_new(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => self.creer_projet_depuis_input(),
            KeyCode::Esc => { self.mode = Mode::Projects; self.input.clear(); }
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'u' {
                    self.input.clear();
                } else if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'o' {
                    if let Some(path) = choisir_dossier() {
                        self.input = path;
                    }
                } else if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.input.push(c);
                }
            }
            KeyCode::Backspace => { self.input.pop(); }
            _ => {}
        }
    }

    fn key_project_delete(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => self.supprimer_projet_confirme(),
            KeyCode::Char('n') | KeyCode::Esc => self.annuler_suppression(),
            _ => {}
        }
    }

    fn key_normal(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => {
                if self.show_help { self.show_help = false; }
                else { self.retour_projets(); }
            }
            KeyCode::Char('p') => self.retour_projets(),
            KeyCode::Char('?') => self.show_help = !self.show_help,
            KeyCode::Char('t') => self.basculer_theme(),
            KeyCode::Char('r') => { self.reload(); self.sync_agents(); self.message = "Rechargé".into(); }
            KeyCode::Left | KeyCode::Char('h') => self.curseur_gauche(),
            KeyCode::Right | KeyCode::Char('l') => self.curseur_droite(),
            KeyCode::Up | KeyCode::Char('k') => self.curseur_haut(),
            KeyCode::Down | KeyCode::Char('j') => self.curseur_bas(),
            KeyCode::Char('H') => self.deplacer_gauche(),
            KeyCode::Char('L') => self.deplacer_droite(),
            KeyCode::Char('a') => self.entrer_insertion(),
            KeyCode::Char('d') => self.supprimer(),
            KeyCode::Enter => self.entrer_detail(),
            _ => {}
        }
    }

    fn key_insert(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => self.confirmer_ajout(),
            KeyCode::Esc => { self.mode = Mode::Normal; self.input.clear(); }
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'u' { self.input.clear(); }
                else if !key.modifiers.contains(KeyModifiers::CONTROL) { self.input.push(c); }
            }
            KeyCode::Backspace => { self.input.pop(); }
            _ => {}
        }
    }

    fn key_detail(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => { self.mode = Mode::Normal; self.message.clear(); }
            KeyCode::Enter => self.envoyer_prompt(),
            KeyCode::Backspace => { self.prompt_input.pop(); }
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'u' {
                    self.prompt_input.clear();
                } else if key.modifiers.contains(KeyModifiers::CONTROL) && c == 's' {
                    if let Some(id) = self.ticket_detail_id() {
                        if let Some(t) = self.tickets.iter().find(|t| t.id == id) {
                            if t.worktree.is_empty() { self.demarrer_worktree(&id); }
                        }
                    }
                } else if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'f' {
                    self.focus_selection();
                } else if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.prompt_input.push(c);
                }
            }
            _ => {}
        }
    }

    fn ticket_sous_souris(&self, mx: u16, my: u16) -> Option<(usize, usize)> {
        for z in &self.hit_zones {
            if mx >= z.rect.x && mx < z.rect.x + z.rect.width
                && my >= z.rect.y && my < z.rect.y + z.rect.height
            {
                return Some((z.col, z.ticket));
            }
        }
        None
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) {
        let (mx, my) = (mouse.column, mouse.row);

        // --- PROJECTS MODE ---
        if self.mode == Mode::Projects {
            match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    for z in &self.project_zones {
                        if mx >= z.rect.x && mx < z.rect.x + z.rect.width
                            && my >= z.rect.y && my < z.rect.y + z.rect.height
                        {
                            self.project_cursor = z.idx;
                            self.selectionner_projet();
                            return;
                        }
                    }
                }
                MouseEventKind::ScrollDown => {
                    if self.project_cursor < self.projects.len().saturating_sub(1) {
                        self.project_cursor += 1;
                    }
                }
                MouseEventKind::ScrollUp => {
                    if self.project_cursor > 0 { self.project_cursor -= 1; }
                }
                _ => {}
            }
            return;
        }

        // --- DETAIL MODE --- ignore clicks (they go to prompt)
        if self.mode == Mode::Detail {
            return;
        }

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.show_context_menu = false;

                // Si on drag déjà, c'est un drop
                if let Some(drag) = self.drag.take() {
                    if let Some((col_target, _)) = self.ticket_sous_souris(mx, my) {
                        if col_target != drag.col_source {
                            let _ = self.db.deplacer_ticket(&drag.ticket_id, COLONNES[col_target]);
                            self.reload();
                            self.message = format!("{} -> {}", drag.ticket_id, COLONNES[col_target]);
                        }
                    }
                    return;
                }

                // Nouveau clic sur un ticket
                if let Some((col, ti)) = self.ticket_sous_souris(mx, my) {
                    self.col_cursor = col;
                    self.ticket_cursor = ti;
                    // Stocker pour drag potentiel
                    if let Some(t) = self.ticket_selectionne() {
                        self.drag = Some(DragState {
                            ticket_id: t.id.clone(),
                            col_source: col,
                        });
                    }
                    self.entrer_detail();
                    return;
                }
            }

            MouseEventKind::Down(MouseButton::Right) => {
                if let Some((col, ti)) = self.ticket_sous_souris(mx, my) {
                    self.col_cursor = col;
                    self.ticket_cursor = ti;
                    self.show_context_menu = true;
                    self.context_cursor = 0;
                    self.mode = Mode::ContextMenu;
                }
            }

            MouseEventKind::Drag(MouseButton::Left) => {
                // Drag actif — visuellement on ne peut pas déplacer le curseur dans ratatui
                // mais on garde l'état pour le drop
            }

            MouseEventKind::Up(MouseButton::Left) => {
                // Drop
                if let Some(drag) = self.drag.take() {
                    if let Some((col_target, _)) = self.ticket_sous_souris(mx, my) {
                        if col_target != drag.col_source {
                            let _ = self.db.deplacer_ticket(&drag.ticket_id, COLONNES[col_target]);
                            self.reload();
                            self.message = format!("{} -> {} (drag)", drag.ticket_id, COLONNES[col_target]);
                        }
                    }
                }
            }

            MouseEventKind::ScrollDown => self.curseur_bas(),
            MouseEventKind::ScrollUp => self.curseur_haut(),
            _ => {}
        }
    }

    pub fn enregistrer_hitzone(&mut self, col: usize, ticket: usize, rect: Rect) {
        self.hit_zones.push(HitZone { col, ticket, rect });
    }
    pub fn reset_hitzones(&mut self) { self.hit_zones.clear(); }
    pub fn enregistrer_projectzone(&mut self, idx: usize, rect: Rect) {
        self.project_zones.push(ProjectZone { idx, rect });
    }
    pub fn reset_projectzones(&mut self) { self.project_zones.clear(); }
}
