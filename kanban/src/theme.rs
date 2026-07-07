use ratatui::style::Color;

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

    pub fn sakura() -> Self {
        Theme {
            name: "Sakura",
            bg: hex(0xfbf3f0),
            bg_alt: hex(0xf3e8e3),
            surface: hex(0xead9d2),
            surface_hi: hex(0xdec4ba),
            fg: hex(0x4a3536),
            fg_dim: hex(0x9c8480),
            accent: hex(0xc56a8a),
            accent_dim: hex(0xa85878),
            green: hex(0x5e9472),
            yellow: hex(0xc2923f),
            red: hex(0xc0504d),
            blue: hex(0x4d7fa8),
            purple: hex(0x8a5fab),
            cyan: hex(0x4a8f99),
            orange: hex(0xc8763f),
            pink: hex(0xc56a8a),
            border: hex(0xddc8c0),
            border_hi: hex(0xc56a8a),
        }
    }

    pub fn nord_light() -> Self {
        Theme {
            name: "Nord Light",
            bg: hex(0xf4f6fa),
            bg_alt: hex(0xe8ecf3),
            surface: hex(0xdce3ed),
            surface_hi: hex(0xcbd5e3),
            fg: hex(0x3b4252),
            fg_dim: hex(0x7b8494),
            accent: hex(0x5e81ac),
            accent_dim: hex(0x4c6c92),
            green: hex(0x5f8c6a),
            yellow: hex(0xc99a3a),
            red: hex(0xbf616a),
            blue: hex(0x5e81ac),
            purple: hex(0x8c6899),
            cyan: hex(0x4d8a9e),
            orange: hex(0xc8744a),
            pink: hex(0xb56a8a),
            border: hex(0xcbd5e3),
            border_hi: hex(0x5e81ac),
        }
    }

    pub fn solarized_light() -> Self {
        Theme {
            name: "Solarized Light",
            bg: hex(0xfdf6e3),
            bg_alt: hex(0xeee8d5),
            surface: hex(0xe3dcc2),
            surface_hi: hex(0xd6cdab),
            fg: hex(0x586e75),
            fg_dim: hex(0x93a1a1),
            accent: hex(0x268bd2),
            accent_dim: hex(0x1e6fa7),
            green: hex(0x719e3f),
            yellow: hex(0xb58900),
            red: hex(0xdc322f),
            blue: hex(0x268bd2),
            purple: hex(0x6c71c4),
            cyan: hex(0x2aa198),
            orange: hex(0xcb4b16),
            pink: hex(0xd33682),
            border: hex(0xd6cdab),
            border_hi: hex(0x268bd2),
        }
    }

    pub fn gruvbox_light() -> Self {
        Theme {
            name: "Gruvbox Light",
            bg: hex(0xfbf1c7),
            bg_alt: hex(0xf2e5bc),
            surface: hex(0xe6d6a4),
            surface_hi: hex(0xd9c78c),
            fg: hex(0x3c3836),
            fg_dim: hex(0x928374),
            accent: hex(0x458588),
            accent_dim: hex(0x3a6f72),
            green: hex(0x79740e),
            yellow: hex(0xb57614),
            red: hex(0x9d0009),
            blue: hex(0x076678),
            purple: hex(0x8f3f71),
            cyan: hex(0x427b58),
            orange: hex(0xaf3a03),
            pink: hex(0xb16286),
            border: hex(0xd9c78c),
            border_hi: hex(0x458588),
        }
    }

    /// Liste ordonnée de tous les thèmes (sombre / clair alternés).
    pub fn all() -> &'static [fn() -> Theme] {
        &[
            Theme::catppuccin,
            Theme::sakura,
            Theme::nord_light,
            Theme::solarized_light,
            Theme::gruvbox_light,
            Theme::dracula,
        ]
    }

    pub fn next(&self) -> Self {
        let themes = Theme::all();
        let idx = themes
            .iter()
            .position(|ctor| ctor().name == self.name)
            .map(|i| (i + 1) % themes.len())
            .unwrap_or(0);
        themes[idx]()
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
