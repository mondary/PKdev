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

pub fn focus_agent(target: &str) -> bool {
    Command::new(bin())
        .args(["agent", "focus", target])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn get_agent_info(target: &str) -> Option<String> {
    let output = Command::new(bin())
        .args(["agent", "explain", target])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            Some(String::from_utf8_lossy(&o.stdout).to_string())
        }
        _ => None,
    }
}

pub fn send_prompt(name: &str, text: &str) -> bool {
    Command::new(bin())
        .args(["agent", "send", name, text])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn start_agent(name: &str, cwd: &str) -> bool {
    // Cibler le workspace/tab où tourne le kanban (hérité des env vars herdr)
    // pour que le pane s'ouvre dans la bonne fenêtre Kaku, pas ailleurs.
    let ws = std::env::var("HERDR_WORKSPACE_ID").ok();
    let tab = std::env::var("HERDR_TAB_ID").ok();

    let mut args: Vec<String> = vec![
        "agent".into(), "start".into(), name.into(),
        "--cwd".into(), cwd.into(),
        "--split".into(), "down".into(),
        "--focus".into(),
    ];
    if let Some(w) = ws {
        args.push("--workspace".into());
        args.push(w);
    }
    if let Some(t) = tab {
        args.push("--tab".into());
        args.push(t);
    }
    args.push("--".into());
    args.push("opencode".into());

    let out = Command::new(bin()).args(&args).output();
    out.map(|o| o.status.success()).unwrap_or(false)
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
