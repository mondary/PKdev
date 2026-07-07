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



pub fn herdr_dispo() -> bool {
    Command::new(bin())
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
