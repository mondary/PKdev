use chrono::{Local, NaiveDateTime, TimeZone};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, Mode, COLONNES};
use crate::theme::Theme;

fn session_age_label(ticket: &crate::db::Ticket) -> Option<String> {
    let started = ticket.session_started_le.as_ref()?;
    let dt = NaiveDateTime::parse_from_str(started, "%Y-%m-%d %H:%M:%S").ok()?;
    let started = Local.from_local_datetime(&dt).single()?;
    let secs = (Local::now() - started).num_seconds().max(0);
    let mins = secs / 60;
    let secs = secs % 60;
    Some(format!("{:02}m{:02}s", mins, secs))
}

pub fn render(frame: &mut Frame, app: &mut App) {
    app.reset_hitzones();
    app.reset_projectzones();
    let area = frame.area();
    let t = app.theme;

    match app.mode {
        Mode::Projects | Mode::ProjectNew => render_projects(frame, app, area, t),
        Mode::DirBrowser => {
            render_projects(frame, app, area, t);
            render_browser(frame, app, area, t);
        }
        Mode::ProjectDelete => {
            render_projects(frame, app, area, t);
            render_confirm_delete(frame, app, area, t);
        }
        Mode::Normal | Mode::Insert => render_board(frame, app, area, t),
        Mode::Detail => {
            render_board(frame, app, area, t);
            render_detail(frame, app, area, t);
        }
        Mode::ContextMenu => {
            render_board(frame, app, area, t);
        }
    }

    if app.mode == Mode::ContextMenu {
        render_context_menu(frame, app, area, t);
    }
    if app.show_help {
        render_aide(frame, app, area, t);
    }
}

fn render_projects(frame: &mut Frame, app: &mut App, area: Rect, t: Theme) {
    if app.mode == Mode::ProjectNew {
        render_project_new(frame, app, area, t);
        return;
    }

    let root = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(24)])
        .split(area);

    let gauche = root[0];
    let sidebar = root[1];

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)])
        .split(gauche);

    // Top bar
    let vue_label = if app.project_view_list { "liste" } else { "tableau" };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" PROJETS", Style::default().fg(t.fg).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  {} projets  ", app.projects.len()), Style::default().fg(t.fg_dim)),
            Span::styled(format!("[vue: {}]", vue_label), Style::default().fg(t.purple)),
        ])).style(Style::default().bg(t.bg_alt)),
        vert[0],
    );

    // Snapshot data
    let pdata: Vec<(String, String, Vec<crate::db::Ticket>)> = app
        .projects
        .iter()
        .map(|p| {
            let tickets: Vec<crate::db::Ticket> = app
                .all_tickets
                .iter()
                .filter(|tk| tk.projet_id == p.id && tk.statut != "done")
                .cloned()
                .collect();
            (p.id.clone(), p.nom.clone(), tickets)
        })
        .collect();

    if app.project_view_list {
        render_projects_list(frame, app, vert[1], t, &pdata);
    } else {
        render_projects_columns(frame, app, vert[1], t, &pdata);
    }

    // Bottom bar
    let bottom = if !app.message.is_empty() {
        Line::from(Span::styled(format!(" {}", app.message), Style::default().fg(t.fg_dim)))
    } else {
        Line::from(Span::styled(
            " [\u{2190}\u{2192}] naviguer  [Entr\u{00e9}e] ouvrir  [a] ajouter  [d] supprimer  [Tab] vue  [t] th\u{00e8}me  [q] quitter",
            Style::default().fg(t.fg_dim),
        ))
    };
    frame.render_widget(Paragraph::new(bottom).style(Style::default().bg(t.bg_alt)), vert[2]);

    // Sidebar
    render_sidebar_projects(frame, app, sidebar, t);
}

fn render_projects_columns(frame: &mut Frame, app: &mut App, area: Rect, t: Theme, pdata: &[(String, String, Vec<crate::db::Ticket>)]) {
    let nb = pdata.len().max(1) as u16;
    let col_rects = Layout::default()
        .direction(Direction::Horizontal)
        .constraints((0..nb).map(|_| Constraint::Percentage(100 / nb as u16)).collect::<Vec<_>>())
        .split(area);

    for (idx, (_id, nom, tickets)) in pdata.iter().enumerate() {
        let r = col_rects[idx];
        let sel = idx == app.project_cursor;
        let header_bg = if sel { t.surface } else { t.bg_alt };

        let header = Line::from(vec![
            Span::styled(
                if sel { " > " } else { "   " },
                Style::default().fg(if sel { t.accent } else { t.fg_dim }),
            ),
            Span::styled(
                nom,
                Style::default()
                    .fg(if sel { t.fg } else { t.fg_dim })
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  {}", tickets.len()), Style::default().fg(t.fg_dim)),
        ]);
        app.enregistrer_projectzone(idx, Rect { x: r.x, y: r.y, width: r.width, height: r.height });
        frame.render_widget(
            Paragraph::new(header).style(Style::default().bg(header_bg)),
            Rect { x: r.x, y: r.y, width: r.width, height: 1 },
        );

        for (ti, ticket) in tickets.iter().enumerate() {
            let y = r.y + 2 + ti as u16;
            if y >= r.y + r.height { break; }
            let couleur = t.status_color(&ticket.statut);
            let dot = if !ticket.agent_id.is_empty() { "\u{25CF} " } else { "  " };
            let age = if ticket.agent_id.is_empty() {
                String::new()
            } else {
                session_age_label(ticket).map(|s| format!("[{}] ", s)).unwrap_or_default()
            };

            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(dot, Style::default().fg(t.green)),
                    Span::styled(age, Style::default().fg(t.fg_dim)),
                    Span::styled(format!("{} ", ticket.id), Style::default().fg(couleur).add_modifier(Modifier::BOLD)),
                    Span::styled(truncate(&ticket.titre, r.width.saturating_sub(24) as usize), Style::default().fg(t.fg_dim)),
                ])).style(Style::default().bg(t.bg)),
                Rect { x: r.x, y, width: r.width, height: 1 },
            );
        }

        let last_y = r.y + 2 + tickets.len() as u16;
        if last_y < r.y + r.height {
            frame.render_widget(
                Block::default().style(Style::default().bg(t.bg)),
                Rect { x: r.x, y: last_y, width: r.width, height: r.y + r.height - last_y },
            );
        }
    }
}

fn render_projects_list(frame: &mut Frame, app: &mut App, area: Rect, t: Theme, pdata: &[(String, String, Vec<crate::db::Ticket>)]) {
    let mut lignes: Vec<Line> = Vec::new();
    lignes.push(Line::raw(" "));

    for (idx, (id, nom, tickets)) in pdata.iter().enumerate() {
        let sel = idx == app.project_cursor;
        let y = area.y + 1 + idx as u16;
        if y >= area.y + area.height { break; }

        let rect = Rect { x: area.x, y, width: area.width, height: 1 };
        app.enregistrer_projectzone(idx, rect);

        let ligne = Line::from(vec![
            Span::styled(if sel { " > " } else { "   " }, Style::default().fg(if sel { t.accent } else { t.fg_dim })),
            Span::styled(format!("{:<14}", id), Style::default().fg(if sel { t.fg } else { t.fg_dim }).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{}", nom), Style::default().fg(if sel { t.fg } else { t.fg_dim })),
            Span::styled(format!("  {} actifs", tickets.len()), Style::default().fg(t.fg_dim)),
        ]);
        frame.render_widget(
            Paragraph::new(ligne).style(Style::default().bg(if sel { t.surface } else { t.bg })),
            rect,
        );
    }
}

fn render_browser(frame: &mut Frame, app: &App, area: Rect, t: Theme) {
    let popup = centrer(area, 60, 70);
    frame.render_widget(Clear, popup);

    let mut lignes: Vec<Line> = Vec::new();

    lignes.push(Line::from(vec![
        Span::styled(" PARCOURIR", Style::default().fg(t.accent).add_modifier(Modifier::BOLD)),
        Span::styled(format!("  {}", app.browser_cwd), Style::default().fg(t.fg_dim)),
    ]));
    lignes.push(Line::raw(" "));

    // Ligne 0 : "Choisir ce dossier"
    {
        let sel = app.browser_cursor == 0;
        lignes.push(Line::from(vec![
            Span::styled(if sel { " > " } else { "   " }, Style::default().fg(if sel { t.green } else { t.fg_dim })),
            Span::styled(
                "✓ Utiliser ce dossier",
                Style::default().fg(if sel { t.green } else { t.fg_dim })
                    .add_modifier(if sel { Modifier::BOLD } else { Modifier::empty() }),
            ),
        ]));
    }

    // Dossiers
    for (i, nom) in app.browser_entries.iter().enumerate() {
        let sel = app.browser_cursor == i + 1;
        lignes.push(Line::from(vec![
            Span::styled(if sel { " > " } else { "   " }, Style::default().fg(if sel { t.accent } else { t.fg_dim })),
            Span::styled("📁 ", Style::default().fg(t.yellow)),
            Span::styled(
                nom,
                Style::default().fg(if sel { t.fg } else { t.fg_dim }),
            ),
        ]));
    }

    lignes.push(Line::raw(" "));

    if app.browser_naming {
        lignes.push(Line::from(vec![
            Span::styled(" Nouveau dossier : ", Style::default().fg(t.green).add_modifier(Modifier::BOLD)),
            Span::styled(&app.input, Style::default().fg(t.fg)),
            Span::styled("\u{2588}", Style::default().fg(t.accent)),
        ]));
        lignes.push(Line::from(Span::styled(
            " [Entrée] créer  [Échap] annuler",
            Style::default().fg(t.fg_dim),
        )));
    } else {
        lignes.push(Line::from(Span::styled(
            " [Entrée] choisir/ouvrir  [h/←] parent  [n] nouveau dossier  [j/k] naviguer  [Échap] annuler",
            Style::default().fg(t.fg_dim),
        )));
    }

    frame.render_widget(
        Paragraph::new(lignes).style(Style::default().bg(t.surface)),
        popup,
    );
}

fn render_confirm_delete(frame: &mut Frame, app: &App, area: Rect, t: Theme) {
    let p = match app.projects.get(app.project_cursor) {
        Some(p) => p,
        None => return,
    };
    let nb = app.nb_tickets_projet(&p.id);

    let popup = centrer(area, 50, 25);
    frame.render_widget(Clear, popup);

    let lignes = vec![
        Line::raw(" "),
        Line::from(vec![
            Span::styled(" Supprimer ", Style::default().fg(t.red)),
            Span::styled(format!("\"{}\"", p.id), Style::default().fg(t.fg).add_modifier(Modifier::BOLD)),
            Span::styled(" ?", Style::default().fg(t.red)),
        ]),
        Line::raw(" "),
        Line::from(Span::styled(
            format!(" {} tickets seront supprim\u{00e9}s.", nb),
            Style::default().fg(t.fg_dim),
        )),
        Line::from(Span::styled(
            format!(" Dossier : {}", p.chemin),
            Style::default().fg(t.fg_dim),
        )),
        Line::raw(" "),
        Line::from(vec![
            Span::styled(" [y] ", Style::default().fg(t.red).add_modifier(Modifier::BOLD)),
            Span::styled("confirmer  ", Style::default().fg(t.fg)),
            Span::styled("[n/Esc] ", Style::default().fg(t.fg_dim)),
            Span::styled("annuler", Style::default().fg(t.fg_dim)),
        ]),
    ];

    frame.render_widget(
        Paragraph::new(lignes).style(Style::default().bg(t.surface)),
        popup,
    );
}

fn render_project_new(frame: &mut Frame, app: &App, area: Rect, t: Theme) {
    let root = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(24)])
        .split(area);

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)])
        .split(root[0]);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" NOUVEAU PROJET", Style::default().fg(t.accent).add_modifier(Modifier::BOLD)),
        ])).style(Style::default().bg(t.bg_alt)),
        vert[0],
    );

    let lignes = vec![
        Line::raw(" "),
        Line::from(vec![
            Span::styled(" Chemin du dossier\n ", Style::default().fg(t.fg_dim)),
        ]),
        Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled(&app.input, Style::default().fg(t.fg)),
            Span::styled("\u{2588}", Style::default().fg(t.accent)),
        ]),
        Line::raw(" "),
        Line::from(Span::styled(
            " [Entr\u{00e9}e] valider  [Ctrl+O] parcourir  [\u{00c9}chap] annuler",
            Style::default().fg(t.fg_dim),
        )),
    ];
    frame.render_widget(
        Paragraph::new(lignes).style(Style::default().bg(t.bg)),
        vert[1],
    );
    frame.render_widget(
        Paragraph::new(Line::raw("")).style(Style::default().bg(t.bg_alt)),
        vert[2],
    );
    render_sidebar_projects(frame, app, root[1], t);
}

fn render_sidebar_projects(frame: &mut Frame, app: &App, area: Rect, t: Theme) {
    let mut lignes: Vec<Line> = Vec::new();

    lignes.push(Line::from(Span::styled(
        " GESTIONNAIRE",
        Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
    )));
    lignes.push(Line::from(Span::styled(
        " D'AGENTS",
        Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
    )));
    lignes.push(Line::raw(" "));
    lignes.push(Line::from(vec![
        Span::styled(" Projets  ", Style::default().fg(t.fg_dim)),
        Span::styled(format!("{}", app.projects.len()), Style::default().fg(t.fg)),
    ]));

    let total_tickets: i64 = app.projects.iter().map(|p| app.nb_tickets_projet(&p.id)).sum();
    lignes.push(Line::from(vec![
        Span::styled(" Tickets  ", Style::default().fg(t.fg_dim)),
        Span::styled(format!("{}", total_tickets), Style::default().fg(t.fg)),
    ]));
    lignes.push(Line::raw(" "));

    lignes.push(Line::from(Span::styled(
        " COMMANDES",
        Style::default().fg(t.fg_dim).add_modifier(Modifier::BOLD),
    )));
    lignes.push(Line::raw(" "));
    for (k, l) in [
        ("Entrée", "ouvrir projet"),
        ("a", "ajouter projet"),
        ("d", "supprimer"),
        ("Tab", "grille / liste"),
        ("j/k", "naviguer"),
        ("clic", "sélectionner"),
        ("t", "thème"),
        ("q", "quitter"),
    ] {
        lignes.push(Line::from(vec![
            Span::styled(format!(" {} ", k), Style::default().fg(t.accent)),
            Span::styled(l, Style::default().fg(t.fg_dim)),
        ]));
    }
    lignes.push(Line::raw(" "));
    lignes.push(Line::from(Span::styled(
        format!(" {}", t.name),
        Style::default().fg(t.purple),
    )));

    frame.render_widget(
        Paragraph::new(lignes).style(Style::default().bg(t.bg_alt)),
        area,
    );
}

fn render_board(frame: &mut Frame, app: &mut App, area: Rect, t: Theme) {
    let root = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(26)])
        .split(area);

    let gauche = root[0];
    let sidebar = root[1];

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)])
        .split(gauche);

    render_topbar(frame, app, vert[0], t);
    render_colonnes(frame, app, vert[1], t);
    render_bottombar(frame, app, vert[2], t);
    render_sidebar(frame, app, sidebar, t);
}

fn render_topbar(frame: &mut Frame, app: &App, area: Rect, t: Theme) {
    let ligne = Line::from(vec![
        Span::styled(
            format!(" {}", app.project_id.to_uppercase()),
            Style::default().fg(t.fg).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {} tickets", app.tickets.len()),
            Style::default().fg(t.fg_dim),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(ligne).style(Style::default().bg(t.bg_alt)),
        area,
    );
}

fn render_colonnes(frame: &mut Frame, app: &mut App, area: Rect, t: Theme) {
    let col_cursor = app.col_cursor;
    let ticket_cursor = app.ticket_cursor;

    let donnees: Vec<(&'static str, Vec<crate::db::Ticket>)> = COLONNES
        .iter()
        .map(|&s| {
            (
                s,
                app.tickets.iter().filter(|tk| tk.statut == s).cloned().collect(),
            )
        })
        .collect();

    let rects = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            (0..COLONNES.len())
                .map(|_| Constraint::Percentage(25))
                .collect::<Vec<_>>(),
        )
        .split(area);

    for (idx, (statut, tickets)) in donnees.iter().enumerate() {
        let r = rects[idx];
        let courant = idx == col_cursor;
        let couleur = t.status_color(statut);

        // En-tête
        let header_bg = if courant { t.surface } else { t.bg_alt };
        let header = Line::from(vec![
            Span::styled(
                format!(" {} ", t.status_icon(statut)),
                Style::default().fg(couleur),
            ),
            Span::styled(
                t.status_label(statut),
                Style::default()
                    .fg(if courant { t.fg } else { t.fg_dim })
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  {}", tickets.len()), Style::default().fg(t.fg_dim)),
        ]);
        frame.render_widget(
            Paragraph::new(header).style(Style::default().bg(header_bg)),
            Rect { x: r.x, y: r.y, width: r.width, height: 1 },
        );

        // Cartes compactes — 1 ligne par ticket
        for (ti, ticket) in tickets.iter().enumerate() {
            let y = r.y + 2 + ti as u16;
            if y >= r.y + r.height {
                break;
            }
            let sel = courant && ti == ticket_cursor;
            let bg = if sel { t.surface_hi } else { t.bg };

            let mut spans = vec![
                Span::styled(
                    format!(" {}", ticket.id),
                    Style::default()
                        .fg(if sel { couleur } else { t.fg_dim })
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {}", truncate(&ticket.titre, r.width.saturating_sub(12) as usize)),
                    Style::default().fg(if sel { t.fg } else { t.fg_dim }),
                ),
            ];

            // Indicateur agent Herdr
            if !ticket.agent_id.is_empty() {
                spans.push(Span::styled(" \u{25CF}", Style::default().fg(t.green)));
            }

            let ligne_rect = Rect { x: r.x, y, width: r.width, height: 1 };
            app.enregistrer_hitzone(idx, ti, ligne_rect);
            frame.render_widget(
                Paragraph::new(Line::from(spans)).style(Style::default().bg(bg)),
                ligne_rect,
            );
        }
    }
}

fn render_sidebar(frame: &mut Frame, app: &App, area: Rect, t: Theme) {
    let mut lignes: Vec<Line> = Vec::new();

    lignes.push(Line::from(Span::styled(
        " AGENTS",
        Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
    )));
    lignes.push(Line::raw(" "));

    // Compteurs
    lignes.push(Line::from(vec![
        Span::styled(" Backlog   ", Style::default().fg(t.fg_dim)),
        Span::styled(format!("{}", app.nb_colonne("backlog")), Style::default().fg(t.fg_dim)),
    ]));
    lignes.push(Line::from(vec![
        Span::styled(" En cours  ", Style::default().fg(t.fg_dim)),
        Span::styled(format!("{}", app.nb_colonne("doing")), Style::default().fg(t.blue)),
    ]));
    lignes.push(Line::from(vec![
        Span::styled(" Review    ", Style::default().fg(t.fg_dim)),
        Span::styled(format!("{}", app.nb_colonne("review")), Style::default().fg(t.yellow)),
    ]));
    lignes.push(Line::raw(" "));

    // Commandes
    lignes.push(Line::from(Span::styled(
        " COMMANDES",
        Style::default().fg(t.fg_dim).add_modifier(Modifier::BOLD),
    )));
    lignes.push(Line::raw(" "));
    for (k, l) in [
        ("Entrée", "rentrer"),
        ("a", "ajouter"),
        ("H/L", "déplacer"),
        ("d", "supprimer"),
        ("t", "thème"),
        ("?", "aide"),
        ("q", "quitter"),
    ] {
        lignes.push(Line::from(vec![
            Span::styled(format!(" {} ", k), Style::default().fg(t.accent)),
            Span::styled(l, Style::default().fg(t.fg_dim)),
        ]));
    }
    lignes.push(Line::raw(" "));
    lignes.push(Line::from(Span::styled(
        format!(" {}", t.name),
        Style::default().fg(t.purple),
    )));

    frame.render_widget(
        Paragraph::new(lignes).style(Style::default().bg(t.bg_alt)),
        area,
    );
}

fn render_bottombar(frame: &mut Frame, app: &App, area: Rect, t: Theme) {
    let ligne = if app.mode == Mode::Insert {
        Line::from(vec![
            Span::styled("> ", Style::default().fg(t.green)),
            Span::styled(&app.input, Style::default().fg(t.fg)),
            Span::styled("\u{2588}", Style::default().fg(t.accent)),
        ])
    } else if !app.message.is_empty() {
        Line::from(Span::styled(
            format!(" {}", app.message),
            Style::default().fg(t.fg_dim),
        ))
    } else {
        Line::from(Span::styled(
            " Entrée : rentrer dans un ticket",
            Style::default().fg(t.fg_dim),
        ))
    };
    frame.render_widget(
        Paragraph::new(ligne).style(Style::default().bg(t.bg_alt)),
        area,
    );
}

fn render_detail(frame: &mut Frame, app: &App, area: Rect, t: Theme) {
    let ticket = match app
        .colonne_courante()
        .get(app.ticket_cursor)
        .copied()
    {
        Some(tk) => tk,
        None => return,
    };

    let popup = centrer(area, 70, 60);
    frame.render_widget(Clear, popup);

    let couleur = t.status_color(&ticket.statut);
    let mut lignes: Vec<Line> = Vec::new();

    // Header
    lignes.push(Line::from(vec![
        Span::styled(
            format!(" {} ", ticket.id),
            Style::default().fg(couleur).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            &ticket.titre,
            Style::default().fg(t.fg).add_modifier(Modifier::BOLD),
        ),
    ]));
    lignes.push(Line::raw(" "));

    // Statut avec navigation visuelle
    let statuts = ["backlog", "doing", "review", "done"];
    let idx = statuts.iter().position(|s| *s == ticket.statut).unwrap_or(0);
    let statut_bar: String = statuts.iter().enumerate()
        .map(|(i, s)| {
            if i == idx {
                format!("[{}]", s)
            } else {
                format!(" {} ", s)
            }
        })
        .collect::<Vec<_>>()
        .join("→");

    lignes.push(Line::from(vec![
        Span::styled(" Statut   ", Style::default().fg(t.fg_dim)),
        Span::styled(&statut_bar, Style::default().fg(couleur)),
    ]));
    lignes.push(Line::from(vec![
        Span::styled("          ", Style::default().fg(t.fg_dim)),
        Span::styled("←/→ ou H/L pour changer", Style::default().fg(t.fg_dim)),
    ]));
    lignes.push(Line::from(vec![
        Span::styled(" Branche  ", Style::default().fg(t.fg_dim)),
        Span::styled(
            if ticket.branche.is_empty() {
                "—".into()
            } else {
                ticket.branche.clone()
            },
            Style::default().fg(t.purple),
        ),
    ]));
    lignes.push(Line::raw(" "));

    // Historique des prompts
    if !app.prompts.is_empty() {
        lignes.push(Line::from(Span::styled(
            " PROMPTS",
            Style::default().fg(t.fg_dim).add_modifier(Modifier::BOLD),
        )));
        lignes.push(Line::raw(" "));
        for p in &app.prompts {
            lignes.push(Line::from(vec![
                Span::styled(" > ", Style::default().fg(t.cyan)),
                Span::styled(&p.texte, Style::default().fg(t.fg)),
            ]));
        }
        lignes.push(Line::raw(" "));
    }

    // Prompt input — always active
    lignes.push(Line::from(Span::styled(
        " ─────────────────────────────────",
        Style::default().fg(t.fg_dim),
    )));
    lignes.push(Line::from(vec![
        Span::styled(" > ", Style::default().fg(t.green).add_modifier(Modifier::BOLD)),
        Span::styled(&app.prompt_input, Style::default().fg(t.fg)),
        Span::styled("\u{2588}", Style::default().fg(t.accent)),
    ]));
    lignes.push(Line::from(Span::styled(
        " [Entrée] envoyer  [←/→] statut  [o] onglet Herdr  [v] basculer agent  [Échap] retour",
        Style::default().fg(t.fg_dim),
    )));

    frame.render_widget(
        Paragraph::new(lignes)
            .style(Style::default().bg(t.surface))
            .wrap(Wrap { trim: true }),
        popup,
    );
}

fn render_aide(frame: &mut Frame, _app: &App, area: Rect, t: Theme) {
    let popup = centrer(area, 50, 50);
    frame.render_widget(Clear, popup);

    let lignes = vec![
        Line::from(Span::styled(
            " AIDE",
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
        )),
        Line::raw(" "),
        Line::from(vec![Span::styled("  hjkl      ", Style::default().fg(t.green)), Span::styled("Naviguer", Style::default().fg(t.fg))]),
        Line::from(vec![Span::styled("  Entrée    ", Style::default().fg(t.cyan)), Span::styled("Rentrer dans un ticket", Style::default().fg(t.fg))]),
        Line::from(vec![Span::styled("  H / L     ", Style::default().fg(t.yellow)), Span::styled("Déplacer le ticket", Style::default().fg(t.fg))]),
        Line::from(vec![Span::styled("  a         ", Style::default().fg(t.cyan)), Span::styled("Ajouter", Style::default().fg(t.fg))]),
        Line::from(vec![Span::styled("  d         ", Style::default().fg(t.red)), Span::styled("Supprimer", Style::default().fg(t.fg))]),
        Line::from(vec![Span::styled("  t         ", Style::default().fg(t.purple)), Span::styled("Thème", Style::default().fg(t.fg))]),
        Line::from(vec![Span::styled("  clic G    ", Style::default().fg(t.orange)), Span::styled("Sélectionner / drag & drop", Style::default().fg(t.fg))]),
        Line::from(vec![Span::styled("  clic D    ", Style::default().fg(t.orange)), Span::styled("Menu contextuel", Style::default().fg(t.fg))]),
        Line::raw(" "),
        Line::from(Span::styled("  ? / Échap pour fermer", Style::default().fg(t.fg_dim))),
    ];

    frame.render_widget(
        Paragraph::new(lignes).style(Style::default().bg(t.surface)),
        popup,
    );
}

fn render_context_menu(frame: &mut Frame, app: &App, area: Rect, t: Theme) {
    let popup = centrer(area, 30, 30);
    frame.render_widget(Clear, popup);

    let options = [
        ("Détails", "Entrée"),
        ("Déplacer ←", "H"),
        ("Déplacer →", "L"),
        ("Supprimer", "d"),
    ];

    let mut lignes = vec![
        Line::from(Span::styled(
            " ACTIONS",
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
        )),
        Line::raw(" "),
    ];

    for (idx, (label, key)) in options.iter().enumerate() {
        let sel = idx == app.context_cursor;
        lignes.push(Line::from(vec![
            Span::styled(
                if sel { " > " } else { "   " },
                Style::default().fg(if sel { t.accent } else { t.fg_dim }),
            ),
            Span::styled(
                *label,
                Style::default()
                    .fg(if sel { t.fg } else { t.fg_dim })
                    .add_modifier(if sel { Modifier::BOLD } else { Modifier::empty() }),
            ),
            Span::styled(
                format!("  [{}]", key),
                Style::default().fg(t.fg_dim),
            ),
        ]));
    }

    lignes.push(Line::raw(" "));
    lignes.push(Line::from(Span::styled(
        " [↑/↓] naviguer  [Entrée] valider  [Échap] fermer",
        Style::default().fg(t.fg_dim),
    )));

    frame.render_widget(
        Paragraph::new(lignes).style(Style::default().bg(t.surface)),
        popup,
    );
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

fn centrer(area: Rect, px: u16, py: u16) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - py) / 2),
            Constraint::Percentage(py),
            Constraint::Percentage((100 - py) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - px) / 2),
            Constraint::Percentage(px),
            Constraint::Percentage((100 - px) / 2),
        ])
        .split(v[1])[1]
}
