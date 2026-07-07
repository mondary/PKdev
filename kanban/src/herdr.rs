use serde::Deserialize;
use std::process::Command;

fn bin() -> String {
    std::env::var("HERDR_BIN_PATH").unwrap_or_else(|_| "herdr".into())
}

#[derive(Deserialize)]
struct HerdrResponse {
    result: HerdrResult,
}

#[derive(Deserialize)]
struct HerdrResult {
    agents: Vec<HerdrAgent>,
}

#[derive(Deserialize, Clone)]
pub struct HerdrAgent {
    pub name: Option<String>,
    pub agent_status: String,
    pub cwd: String,
    pub pane_id: String,
    pub agent: Option<String>,
}

impl HerdrAgent {
    pub fn etat_court(&self) -> &str {
        match self.agent_status.as_str() {
            "working" => "working",
            "blocked" => "blocked",
            "idle" => "idle",
            _ => "unknown",
        }
    }

    pub fn display_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| {
            self.agent.clone().unwrap_or_else(|| "unknown".into())
        })
    }
}

pub fn list_agents() -> Vec<HerdrAgent> {
    let out = Command::new(bin())
        .args(["agent", "list"])
        .output();

    match out {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            serde_json::from_str::<HerdrResponse>(&text)
                .map(|r| r.result.agents)
                .unwrap_or_default()
        }
        _ => Vec::new(),
    }
}

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

pub fn trouver_ou_creer_workspace(label: &str, cwd: &str) -> Option<String> {
    // Chercher un workspace existant avec ce label
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

    // Créer le workspace s'il n'existe pas
    let out = Command::new(bin())
        .args(["workspace", "create", "--label", label, "--cwd", cwd])
        .output();

    if let Ok(o) = out {
        if o.status.success() {
            let text = String::from_utf8_lossy(&o.stdout);
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(id) = v.get("result")
                    .and_then(|r| r.get("workspace"))
                    .and_then(|w| w.get("workspace_id"))
                    .and_then(|i| i.as_str())
                {
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

pub fn focus_agent(target: &str) -> bool {
    Command::new(bin())
        .args(["agent", "focus", target])
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

pub fn start_agent(name: &str, cwd: &str) -> bool {
    // Trouver ou créer un workspace dédié au projet
    let label = std::path::Path::new(cwd)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "PKdev".into());

    if let Some(ws_id) = trouver_ou_creer_workspace(&label, cwd) {
        let args: Vec<String> = vec![
            "agent".into(), "start".into(), name.into(),
            "--cwd".into(), cwd.into(),
            "--workspace".into(), ws_id,
            "--split".into(), "down".into(),
            "--focus".into(),
            "--".into(),
            "opencode".into(),
        ];
        return Command::new(bin()).args(&args).output()
            .map(|o| o.status.success())
            .unwrap_or(false);
    }

    false
}

pub fn stop_agent(target: &str) -> bool {
    Command::new(bin())
        .args(["pane", "close", target])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn herdr_dispo() -> bool {
    Command::new(bin())
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
