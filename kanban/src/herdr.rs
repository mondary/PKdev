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
    pub name: String,
    pub state: String,
}

impl HerdrAgent {
    pub fn etat_court(&self) -> &str {
        match self.state.as_str() {
            "working" => "working",
            "blocked" => "blocked",
            "idle" => "idle",
            _ => "unknown",
        }
    }
}

pub fn list_agents() -> Vec<HerdrAgent> {
    let out = Command::new(bin())
        .args(["agent", "list", "--json"])
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

pub fn focus_agent(name: &str) -> bool {
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

pub fn start_agent(name: &str, cwd: &str) -> bool {
    let out = Command::new(bin())
        .args([
            "agent", "start", name,
            "--cwd", cwd,
            "--split", "right",
            "--",
            "opencode",
        ])
        .output();
    out.map(|o| o.status.success()).unwrap_or(false)
}

pub fn create_worktree(branch: &str, label: &str) -> Option<String> {
    let out = Command::new(bin())
        .args([
            "worktree", "create",
            "--branch", branch,
            "--label", label,
            "--json",
        ])
        .output();

    match out {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            serde_json::from_str::<serde_json::Value>(&text)
                .ok()
                .and_then(|v| {
                    v.get("result")
                        .and_then(|r| r.get("path"))
                        .and_then(|p| p.as_str())
                        .map(|s| s.to_string())
                })
        }
        _ => None,
    }
}

pub fn herdr_dispo() -> bool {
    Command::new(bin())
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
