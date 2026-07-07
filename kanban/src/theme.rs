use ratatui::style::Color;

pub enum ThemeName {
    Dracula,
    Catppuccin,
}

#[derive(Clone, Copy)]
pub struct Theme {
    pub name: &'static str,
    pub bg: Color,
    pub bg_alt: Color,
    pub surface: Color,
    pub surface_hi: Color,
    pub fg: Color,
    pub fg_dim: Color,
    pub accent: Color,
    pub accent_dim: Color,
    pub green: Color,
    pub yellow: Color,
    pub red: Color,
    pub blue: Color,
    pub purple: Color,
    pub cyan: Color,
    pub orange: Color,
    pub pink: Color,
    pub border: Color,
    pub border_hi: Color,
}

fn hex(h: u32) -> Color {
    Color::Rgb((h >> 16) as u8, (h >> 8) as u8, h as u8)
}

impl Theme {
    pub fn dracula() -> Self {
        Theme {
            name: "Dracula",
            bg: hex(0x282a36),
            bg_alt: hex(0x21222c),
            surface: hex(0x44475a),
            surface_hi: hex(0x565d80),
            fg: hex(0xf8f8f2),
            fg_dim: hex(0x6272a4),
            accent: hex(0xbd93f9),
            accent_dim: hex(0x8b8bcb),
            green: hex(0x50fa7b),
            yellow: hex(0xf1fa8c),
            red: hex(0xff5555),
            blue: hex(0x8be9fd),
            purple: hex(0xbd93f9),
            cyan: hex(0x8be9fd),
            orange: hex(0xffb86c),
            pink: hex(0xff79c6),
            border: hex(0x44475a),
            border_hi: hex(0xff79c6),
        }
    }

    pub fn catppuccin() -> Self {
        Theme {
            name: "Catppuccin Mocha",
            bg: hex(0x1e1e2e),
            bg_alt: hex(0x181825),
            surface: hex(0x313244),
            surface_hi: hex(0x45475a),
            fg: hex(0xcdd6f4),
            fg_dim: hex(0x6c7086),
            accent: hex(0x89b4fa),
            accent_dim: hex(0x74a8e0),
            green: hex(0xa6e3a1),
            yellow: hex(0xf9e2af),
            red: hex(0xf38ba8),
            blue: hex(0x89b4fa),
            purple: hex(0xcba6f7),
            cyan: hex(0x94e2d5),
            orange: hex(0xfab387),
            pink: hex(0xf5c2e7),
            border: hex(0x313244),
            border_hi: hex(0xf5c2e7),
        }
    }

    pub fn next(&self) -> Self {
        match self.name {
            "Dracula" => Theme::catppuccin(),
            _ => Theme::dracula(),
        }
    }

    pub fn status_color(&self, statut: &str) -> Color {
        match statut {
            "backlog" => self.fg_dim,
            "doing" => self.blue,
            "review" => self.yellow,
            "done" => self.green,
            _ => self.fg,
        }
    }

    pub fn status_icon(&self, statut: &str) -> &str {
        match statut {
            "backlog" => "\u{25CB}",
            "doing" => "\u{25D0}",
            "review" => "\u{25D1}",
            "done" => "\u{25CF}",
            _ => "\u{25CB}",
        }
    }

    pub fn status_label(&self, statut: &str) -> &str {
        match statut {
            "backlog" => "BACKLOG",
            "doing" => "EN COURS",
            "review" => "REVIEW",
            "done" => "TERMINÉ",
            _ => "?",
        }
    }
}
