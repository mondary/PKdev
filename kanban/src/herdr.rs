use std::process::Command;

fn kaku() -> String {
    "/Applications/Kaku.app/Contents/MacOS/kaku".into()
}

pub fn herdr_dispo() -> bool {
    std::path::Path::new(&kaku()).exists()
}

/// Lance opencode dans un nouvel onglet Kaku. Retourne le pane-id.
pub fn start_agent(name: &str, cwd: &str) -> Option<String> {
    let result = Command::new(&kaku())
        .args([
            "cli", "spawn",
            "--pane-name", name,
            "--cwd", cwd,
            "--", "opencode",
        ])
        .output();

    match result {
        Ok(o) if o.status.success() => {
            let pane_id = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if pane_id.is_empty() { None } else { Some(pane_id) }
        }
        _ => None,
    }
}

pub fn stop_agent(pane_id: &str) -> bool {
    Command::new(&kaku())
        .args(["cli", "kill-pane", "--pane-id", pane_id])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn focus_agent(pane_id: &str) -> bool {
    Command::new(&kaku())
        .args(["cli", "activate-pane", "--pane-id", pane_id])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn send_prompt(pane_id: &str, text: &str) -> bool {
    let full_text = format!("{}\r", text);
    Command::new(&kaku())
        .args(["cli", "send-text", "--pane-id", pane_id, "--no-paste", &full_text])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
