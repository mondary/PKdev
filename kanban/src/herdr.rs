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

#[derive(Deserialize)]
struct TabListResponse {
    result: TabListResult,
}

#[derive(Deserialize)]
struct TabListResult {
    tabs: Vec<HerdrTab>,
}

#[derive(Deserialize, Clone)]
pub struct HerdrTab {
    pub tab_id: String,
    pub label: String,
}

/// Trouve ou crée un workspace herdr pour le projet.
pub fn trouver_ou_creer_workspace(label: &str, cwd: &str) -> Option<String> {
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

fn tabs_for_workspace(workspace_id: &str) -> Vec<HerdrTab> {
    let out = Command::new(bin())
        .args(["tab", "list", "--workspace", workspace_id])
        .output();

    match out {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            serde_json::from_str::<TabListResponse>(&text)
                .map(|r| r.result.tabs)
                .unwrap_or_default()
        }
        _ => Vec::new(),
    }
}

fn trouver_ou_creer_tab(workspace_id: &str, cwd: &str, label: &str) -> Option<String> {
    for tab in tabs_for_workspace(workspace_id) {
        if tab.label == label {
            return Some(tab.tab_id);
        }
    }

    let _ = Command::new(bin())
        .args(["tab", "create", "--workspace", workspace_id, "--cwd", cwd, "--label", label, "--focus"])
        .output();

    for tab in tabs_for_workspace(workspace_id) {
        if tab.label == label {
            return Some(tab.tab_id);
        }
    }
    None
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
    pub tab_id: String,
    pub workspace_id: String,
}

#[derive(Deserialize, Clone)]
pub struct HerdrAgentsResult {
    pub agents: Vec<HerdrAgent>,
}

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

pub fn start_agent(name: &str, cwd: &str, workspace_id: &str) -> Option<String> {
    let tab_id = trouver_ou_creer_tab(workspace_id, cwd, name)?;

    let agents = list_agents();
    for a in &agents.agents {
        if a.name.as_deref() == Some(name) && a.workspace_id == workspace_id {
            let _ = Command::new(bin())
                .args(["tab", "focus", &tab_id])
                .output();
            return Some(name.to_string());
        }
    }

    let result = Command::new(bin())
        .args([
            "agent", "start", name,
            "--cwd", cwd,
            "--workspace", workspace_id,
            "--tab", &tab_id,
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

pub fn focus_agent(name: &str) -> bool {
    let agents = list_agents();
    for a in &agents.agents {
        if a.name.as_deref() == Some(name) {
            return Command::new(bin())
                .args(["tab", "focus", &a.tab_id])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
        }
    }
    Command::new(bin())
        .args(["agent", "focus", name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn send_prompt(name: &str, text: &str) -> bool {
    Command::new(bin())
        .args(["agent", "send", name, text])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn stop_agent(name: &str) -> bool {
    let agents = list_agents();
    for a in &agents.agents {
        if a.name.as_deref() == Some(name) {
            return Command::new(bin())
                .args(["tab", "close", &a.tab_id])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
        }
    }
    false
}

pub fn agent_status(name: &str) -> String {
    let agents = list_agents();
    for a in &agents.agents {
        if a.name.as_deref() == Some(name) {
            return a.agent_status.clone();
        }
    }
    "unknown".to_string()
}
