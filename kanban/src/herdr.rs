use serde::Deserialize;
use std::process::Command;

fn bin() -> String {
    std::env::var("HERDR_BIN_PATH").unwrap_or_else(|_| "herdr".into())
}

pub fn herdr_dispo() -> bool {
    Command::new(bin())
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// =====================================================
//  WORKSPACES (= un workspace par projet)
// =====================================================

#[derive(Deserialize)]
struct WorkspaceListResponse {
    result: WorkspaceListResult,
}

#[derive(Deserialize)]
struct WorkspaceListResult {
    workspaces: Vec<HerdrWorkspace>,
}

#[derive(Deserialize, Clone)]
pub struct HerdrWorkspace {
    pub workspace_id: String,
    pub label: String,
}

/// Trouve ou crée un workspace herdr pour le projet.
/// Retourne le workspace_id.
pub fn trouver_ou_creer_workspace(label: &str, cwd: &str) -> Option<String> {
    // Chercher un workspace existant
    if let Ok(o) = Command::new(bin()).args(["workspace", "list"]).output() {
        if o.status.success() {
            let text = String::from_utf8_lossy(&o.stdout);
            if let Ok(resp) = serde_json::from_str::<WorkspaceListResponse>(&text) {
                for ws in &resp.result.workspaces {
                    if ws.label == label {
                        return Some(ws.workspace_id.clone());
                    }
                }
            }
        }
    }

    // Créer le workspace
    let out = Command::new(bin())
        .args(["workspace", "create", "--label", label, "--cwd", cwd, "--focus"])
        .output();

    if let Ok(o) = out {
        if o.status.success() {
            let text = String::from_utf8_lossy(&o.stdout);
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(id) = v.pointer("/result/workspace/workspace_id").and_then(|i| i.as_str()) {
                    return Some(id.to_string());
                }
            }
        }
    }
    None
}

pub fn focus_workspace(workspace_id: &str) -> bool {
    Command::new(bin())
        .args(["workspace", "focus", workspace_id])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// =====================================================
//  AGENTS (= un agent opencode par ticket)
// =====================================================

#[derive(Deserialize)]
struct AgentListResponse {
    result: AgentListResult,
}

#[derive(Deserialize)]
struct AgentListResult {
    agents: Vec<HerdrAgent>,
}

#[derive(Deserialize, Clone)]
pub struct HerdrAgent {
    pub name: Option<String>,
    pub agent_status: String,
    pub cwd: String,
    pub pane_id: String,
    pub workspace_id: String,
}

#[derive(Deserialize, Clone)]
pub struct HerdrAgentsResult {
    pub agents: Vec<HerdrAgent>,
}

/// Liste tous les agents actifs dans herdr
pub fn list_agents() -> HerdrAgentsResult {
    let out = Command::new(bin()).args(["agent", "list"]).output();

    match out {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            if let Ok(resp) = serde_json::from_str::<AgentListResponse>(&text) {
                return HerdrAgentsResult { agents: resp.result.agents };
            }
        }
        _ => {}
    }
    HerdrAgentsResult { agents: Vec::new() }
}

/// Démarre opencode pour un ticket dans le workspace du projet.
/// Si l'agent existe déjà (même nom), ne fait rien (réutilisation = pas de RAM dupliquée).
/// Retourne le nom de l'agent.
pub fn start_agent(name: &str, cwd: &str, workspace_id: &str) -> Option<String> {
    // Vérifier si l'agent existe déjà dans ce workspace
    let agents = list_agents();
    for a in &agents.agents {
        if a.name.as_deref() == Some(name) && a.workspace_id == workspace_id {
            // Agent existe déjà → on le réutilise
            return Some(name.to_string());
        }
    }

    // Créer l'agent dans le workspace
    let result = Command::new(bin())
        .args([
            "agent", "start", name,
            "--cwd", cwd,
            "--workspace", workspace_id,
            "--split", "right",
            "--focus",
            "--", "opencode",
        ])
        .output();

    match result {
        Ok(o) if o.status.success() => Some(name.to_string()),
        Ok(o) => {
            eprintln!("start_agent: {}", String::from_utf8_lossy(&o.stderr));
            None
        }
        Err(_) => None,
    }
}

/// Focus sur un agent par son nom
pub fn focus_agent(name: &str) -> bool {
    Command::new(bin())
        .args(["agent", "focus", name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Envoie du texte à un agent
pub fn send_prompt(name: &str, text: &str) -> bool {
    Command::new(bin())
        .args(["agent", "send", name, text])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Arrête un agent
pub fn stop_agent(name: &str) -> bool {
    Command::new(bin())
        .args(["agent", "focus", name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    // Fermer le pane de l'agent
    && Command::new(bin())
        .args(["pane", "close", name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Statut d'un agent (working / idle / blocked / unknown)
pub fn agent_status(name: &str) -> String {
    let agents = list_agents();
    for a in &agents.agents {
        if a.name.as_deref() == Some(name) {
            return a.agent_status.clone();
        }
    }
    "unknown".to_string()
}
