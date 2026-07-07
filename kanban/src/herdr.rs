use serde::Deserialize;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

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

fn ensure_server() {
    let running = Command::new(bin())
        .args(["status", "server"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("status: running"))
        .unwrap_or(false);

    if running {
        return;
    }

    let _ = Command::new(bin())
        .arg("server")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    thread::sleep(Duration::from_millis(250));
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
pub fn trouver_ou_creer_workspace(label: &str, cwd: &str) -> Option<String> {
    ensure_server();

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
    ensure_server();

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
    pub agent: Option<String>,
    pub agent_status: String,
    pub cwd: String,
    pub pane_id: String,
    pub tab_id: String,
    pub workspace_id: String,
}

#[derive(Deserialize)]
struct PaneListResponse {
    result: PaneListResult,
}

#[derive(Deserialize)]
struct PaneListResult {
    panes: Vec<HerdrPane>,
}

#[derive(Deserialize, Clone)]
pub struct HerdrPane {
    pub pane_id: String,
    pub tab_id: String,
    pub workspace_id: String,
    pub agent: Option<String>,
    pub label: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct HerdrAgentsResult {
    pub agents: Vec<HerdrAgent>,
}

pub struct StartedAgent {
    pub name: String,
    pub created: bool,
}

pub fn list_agents() -> HerdrAgentsResult {
    ensure_server();

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

fn list_panes(workspace_id: &str) -> Vec<HerdrPane> {
    ensure_server();

    let out = Command::new(bin())
        .args(["pane", "list", "--workspace", workspace_id])
        .output();

    match out {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            serde_json::from_str::<PaneListResponse>(&text)
                .map(|r| r.result.panes)
                .unwrap_or_default()
        }
        _ => Vec::new(),
    }
}

fn close_empty_siblings(agent: &HerdrAgent) {
    for pane in list_panes(&agent.workspace_id) {
        if pane.tab_id == agent.tab_id
            && pane.pane_id != agent.pane_id
            && pane.agent.is_none()
            && pane.label.as_deref().unwrap_or_default().is_empty()
        {
            let _ = Command::new(bin())
                .args(["pane", "close", &pane.pane_id])
                .output();
        }
    }
}

fn find_agent(name: &str, workspace_id: &str) -> Option<HerdrAgent> {
    list_agents()
        .agents
        .into_iter()
        .find(|a| {
            a.name.as_deref() == Some(name)
                && a.workspace_id == workspace_id
                && a.agent.as_deref() == Some("opencode")
        })
}

fn wait_agent_ready(name: &str, workspace_id: Option<&str>) -> Option<HerdrAgent> {
    for _ in 0..40 {
        let agents = list_agents();
        for agent in agents.agents {
            if agent.name.as_deref() == Some(name)
                && agent.agent.as_deref() == Some("opencode")
                && workspace_id.map(|id| agent.workspace_id == id).unwrap_or(true)
            {
                return Some(agent);
            }
        }
        thread::sleep(Duration::from_millis(250));
    }
    None
}

fn pane_text(pane_id: &str) -> String {
    let out = Command::new(bin())
        .args([
            "pane",
            "read",
            pane_id,
            "--source",
            "recent-unwrapped",
            "--lines",
            "30",
            "--format",
            "text",
        ])
        .output();

    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => String::new(),
    }
}

fn wait_opencode_ui_ready(pane_id: &str) -> bool {
    for _ in 0..40 {
        let text = pane_text(pane_id);
        if text.contains("/status") || text.contains("OpenCode") || text.contains("opencode") {
            return true;
        }
        thread::sleep(Duration::from_millis(250));
    }
    false
}

pub fn start_agent(name: &str, cwd: &str, workspace_id: &str) -> Option<StartedAgent> {
    let agents = list_agents();
    for a in &agents.agents {
        if a.name.as_deref() == Some(name) && a.workspace_id == workspace_id {
            if a.agent.as_deref() != Some("opencode") {
                let _ = Command::new(bin())
                    .args(["tab", "focus", &a.tab_id])
                    .output();
                let _ = Command::new(bin())
                    .args(["pane", "run", &a.pane_id, "opencode"])
                    .output();

                if let Some(agent) = wait_agent_ready(name, Some(workspace_id)) {
                    close_empty_siblings(&agent);
                    return Some(StartedAgent { name: name.to_string(), created: true });
                }

                return None;
            }

            let _ = Command::new(bin())
                .args(["tab", "focus", &a.tab_id])
                .output();
            close_empty_siblings(a);
            return Some(StartedAgent { name: name.to_string(), created: false });
        }
    }

    let result = Command::new(bin())
        .args([
            "agent", "start", name,
            "--cwd", cwd,
            "--workspace", workspace_id,
            "--focus",
            "--", "opencode",
        ])
        .output();

    match result {
        Ok(o) if o.status.success() => {
            thread::sleep(Duration::from_millis(150));
            if let Some(agent) = find_agent(name, workspace_id) {
                close_empty_siblings(&agent);
            }
            Some(StartedAgent { name: name.to_string(), created: true })
        }
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
    ensure_server();

    let Some(agent) = wait_agent_ready(name, None) else {
        return false;
    };

    if !wait_opencode_ui_ready(&agent.pane_id) {
        return false;
    }

    let sent = Command::new(bin())
        .args(["agent", "send", name, text])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !sent {
        return false;
    }

    for _ in 0..10 {
        if pane_text(&agent.pane_id).contains(text) {
            return true;
        }
        thread::sleep(Duration::from_millis(200));
    }

    false
}

pub fn stop_agent(name: &str) -> bool {
    ensure_server();

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
