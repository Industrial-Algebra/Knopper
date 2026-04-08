use crate::{Color, Padding, Scene, SizeConstraint, Style};

const PANEL_HEADER_ROWS: u16 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelFill {
    Plain,
    Grid,
    DenseGrid,
    Bands,
    Dots,
}

#[must_use]
pub fn truncate_text(text: &str, width: u16) -> String {
    let max = usize::from(width.max(1));
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max {
        return text.to_string();
    }
    if max <= 3 {
        return ".".repeat(max);
    }
    let mut out: String = chars.into_iter().take(max.saturating_sub(3)).collect();
    out.push_str("...");
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresenceTone {
    Local,
    Collaborator,
    Passive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresenceCue {
    pub name: &'static str,
    pub label: &'static str,
    pub tone: PresenceTone,
}

#[must_use]
pub fn focus_style() -> Style {
    Style::PLAIN.fg(Color::Ansi(6)).bold()
}

#[must_use]
pub fn surface_style(active: bool) -> Style {
    if active {
        focus_style()
    } else {
        Style::PLAIN.fg(Color::Ansi(8))
    }
}

#[must_use]
pub fn section_title_style(active: bool) -> Style {
    if active {
        focus_style()
    } else {
        Style::PLAIN.fg(Color::Ansi(7)).bold()
    }
}

#[must_use]
pub fn presence_style(tone: PresenceTone) -> Style {
    match tone {
        PresenceTone::Local => focus_style(),
        PresenceTone::Collaborator => Style::PLAIN.fg(Color::Ansi(3)).bold(),
        PresenceTone::Passive => Style::PLAIN.fg(Color::Ansi(8)),
    }
}

#[must_use]
pub fn presence_strip<Msg>(base: u64, cues: &[PresenceCue]) -> Scene<Msg> {
    Scene::row(
        base,
        cues.iter()
            .enumerate()
            .flat_map(|(index, cue)| {
                let id =
                    base.saturating_add(u64::try_from(index).unwrap_or(u64::MAX).saturating_mul(3));
                let mut nodes = vec![
                    Scene::text(id.saturating_add(1), cue.name)
                        .with_style(presence_style(cue.tone)),
                    Scene::text(id.saturating_add(2), format!(":{}", cue.label))
                        .with_style(Style::PLAIN.fg(Color::Ansi(8))),
                ];
                if index + 1 != cues.len() {
                    nodes.push(Scene::text(id.saturating_add(3), "  "));
                }
                nodes
            })
            .collect::<Vec<_>>(),
    )
}

#[must_use]
pub fn panel_header<Msg>(
    base: u64,
    width: u16,
    title: &str,
    subtitle: &str,
    cues: &[PresenceCue],
    active: bool,
) -> Scene<Msg> {
    Scene::column(
        base,
        vec![
            Scene::text(base.saturating_add(1), truncate_text(title, width))
                .with_style(section_title_style(active)),
            Scene::text(base.saturating_add(2), truncate_text(subtitle, width))
                .with_style(Style::PLAIN.fg(Color::Ansi(8))),
            Scene::sized(
                base.saturating_add(3),
                SizeConstraint::width(width),
                Scene::viewport(
                    base.saturating_add(4),
                    presence_strip(base.saturating_add(5), cues),
                ),
            ),
        ],
    )
}

fn panel_fill_row(fill: PanelFill, width: u16, row: usize) -> String {
    let width = usize::from(width.max(1));
    match fill {
        PanelFill::Plain => " ".repeat(width),
        PanelFill::Grid => {
            let mut line = String::with_capacity(width);
            for column in 0..width {
                let major_row = row.is_multiple_of(4);
                let major_col = column.is_multiple_of(8);
                let minor_row = row % 4 == 2;
                let minor_col = column % 8 == 4;
                line.push(if (major_row && minor_col) || (minor_row && major_col) {
                    '·'
                } else {
                    ' '
                });
            }
            line
        }
        PanelFill::DenseGrid => {
            let mut line = String::with_capacity(width);
            for column in 0..width {
                let major_row = row.is_multiple_of(3);
                let major_col = column.is_multiple_of(6);
                let minor_row = row % 3 == 1;
                let minor_col = column % 6 == 3;
                line.push(if (major_row && minor_col) || (minor_row && major_col) {
                    '·'
                } else if major_row && major_col {
                    '•'
                } else {
                    ' '
                });
            }
            line
        }
        PanelFill::Bands => {
            let ch = if row % 4 == 1 { '·' } else { ' ' };
            std::iter::repeat_n(ch, width).collect()
        }
        PanelFill::Dots => {
            let mut line = String::with_capacity(width);
            for column in 0..width {
                let major_row = row.is_multiple_of(5);
                let major_col = column.is_multiple_of(10);
                let minor_row = row % 5 == 2;
                let minor_col = column % 10 == 5;
                line.push(if (major_row && major_col) || (minor_row && minor_col) {
                    '·'
                } else {
                    ' '
                });
            }
            line
        }
    }
}

fn panel_fill_style(fill: PanelFill) -> Style {
    match fill {
        PanelFill::Plain => Style::PLAIN.bg(Color::Ansi(0)),
        PanelFill::Grid => Style::PLAIN.fg(Color::Ansi(8)).bg(Color::Ansi(0)),
        PanelFill::DenseGrid => Style::PLAIN.fg(Color::Ansi(8)).bg(Color::Ansi(0)).bold(),
        PanelFill::Bands => Style::PLAIN.fg(Color::Ansi(8)).bg(Color::Ansi(0)),
        PanelFill::Dots => Style::PLAIN.fg(Color::Ansi(6)).bg(Color::Ansi(0)),
    }
}

fn active_panel_fill(fill: PanelFill) -> PanelFill {
    match fill {
        PanelFill::Plain => PanelFill::Grid,
        PanelFill::Bands => PanelFill::Grid,
        PanelFill::Dots => PanelFill::Grid,
        PanelFill::Grid => PanelFill::DenseGrid,
        PanelFill::DenseGrid => PanelFill::DenseGrid,
    }
}

#[must_use]
pub fn panel_background<Msg>(base: u64, width: u16, height: u16, fill: PanelFill) -> Scene<Msg> {
    Scene::column(
        base,
        (0..usize::from(height.max(1)))
            .map(|offset| {
                Scene::text(
                    base.saturating_add(1 + u64::try_from(offset).unwrap_or(u64::MAX)),
                    panel_fill_row(fill, width, offset),
                )
                .with_style(panel_fill_style(fill))
            })
            .collect::<Vec<_>>(),
    )
}

#[must_use]
pub fn panel_chrome<Msg>(
    base: u64,
    width: u16,
    height: u16,
    header: Scene<Msg>,
    body: Scene<Msg>,
    active: bool,
    fill: PanelFill,
) -> Scene<Msg> {
    let body_height = height.saturating_sub(PANEL_HEADER_ROWS).max(1);
    let fill = if active {
        active_panel_fill(fill)
    } else {
        fill
    };
    Scene::border(
        base,
        Scene::padding(
            base.saturating_add(1),
            Padding::all(1),
            Scene::sized(
                base.saturating_add(2),
                SizeConstraint::new(Some(width), Some(height)),
                Scene::column(
                    base.saturating_add(3),
                    vec![
                        header,
                        Scene::sized(
                            base.saturating_add(4),
                            SizeConstraint::new(Some(width), Some(body_height)),
                            Scene::viewport(
                                base.saturating_add(5),
                                Scene::stack(
                                    base.saturating_add(6),
                                    vec![
                                        panel_background(
                                            base.saturating_add(7),
                                            width,
                                            body_height,
                                            fill,
                                        ),
                                        body,
                                    ],
                                ),
                            ),
                        ),
                    ],
                ),
            ),
        ),
    )
    .with_style(surface_style(active))
}

#[must_use]
pub fn surface_panel<Msg>(
    base: u64,
    width: u16,
    title: &str,
    subtitle: &str,
    cues: &[PresenceCue],
    body: Scene<Msg>,
    active: bool,
) -> Scene<Msg> {
    surface_panel_with_fill(
        base,
        width,
        title,
        subtitle,
        cues,
        body,
        active,
        PanelFill::Grid,
    )
}

#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn surface_panel_with_fill<Msg>(
    base: u64,
    width: u16,
    title: &str,
    subtitle: &str,
    cues: &[PresenceCue],
    body: Scene<Msg>,
    active: bool,
    fill: PanelFill,
) -> Scene<Msg> {
    bounded_surface_panel_with_fill(
        base,
        width,
        PANEL_HEADER_ROWS.saturating_add(1).saturating_add(8),
        title,
        subtitle,
        cues,
        body,
        active,
        fill,
    )
}

#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn bounded_surface_panel<Msg>(
    base: u64,
    width: u16,
    height: u16,
    title: &str,
    subtitle: &str,
    cues: &[PresenceCue],
    body: Scene<Msg>,
    active: bool,
) -> Scene<Msg> {
    bounded_surface_panel_with_fill(
        base,
        width,
        height,
        title,
        subtitle,
        cues,
        body,
        active,
        PanelFill::Grid,
    )
}

#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn bounded_surface_panel_with_fill<Msg>(
    base: u64,
    width: u16,
    height: u16,
    title: &str,
    subtitle: &str,
    cues: &[PresenceCue],
    body: Scene<Msg>,
    active: bool,
    fill: PanelFill,
) -> Scene<Msg> {
    panel_chrome(
        base,
        width,
        height,
        panel_header(
            base.saturating_add(20),
            width,
            title,
            subtitle,
            cues,
            active,
        ),
        body,
        active,
        fill,
    )
}

#[must_use]
pub fn split_columns<Msg>(
    base: u64,
    left_width: u16,
    right_width: u16,
    left: Scene<Msg>,
    right: Scene<Msg>,
) -> Scene<Msg> {
    Scene::row(
        base,
        vec![
            Scene::sized(
                base.saturating_add(1),
                SizeConstraint::width(left_width),
                left,
            ),
            Scene::text(base.saturating_add(2), "  "),
            Scene::sized(
                base.saturating_add(3),
                SizeConstraint::width(right_width),
                right,
            ),
        ],
    )
}

#[must_use]
pub fn master_detail<Msg>(
    base: u64,
    width: u16,
    list_height: u16,
    detail_height: u16,
    list: Scene<Msg>,
    detail: Scene<Msg>,
) -> Scene<Msg> {
    Scene::sized(
        base,
        SizeConstraint::width(width),
        Scene::column(
            base.saturating_add(1),
            vec![
                Scene::sized(
                    base.saturating_add(2),
                    SizeConstraint::new(Some(width), Some(list_height)),
                    Scene::viewport(base.saturating_add(3), list),
                ),
                Scene::sized(
                    base.saturating_add(4),
                    SizeConstraint::new(Some(width), Some(detail_height)),
                    Scene::viewport(base.saturating_add(5), detail),
                ),
            ],
        ),
    )
}

#[must_use]
pub fn app_shell<Msg>(
    base: u64,
    width: u16,
    height: u16,
    title: &str,
    header: Scene<Msg>,
    body: Scene<Msg>,
    footer: Scene<Msg>,
) -> Scene<Msg> {
    Scene::border(
        base,
        Scene::sized(
            base.saturating_add(1),
            SizeConstraint::new(Some(width), Some(height)),
            Scene::column(
                base.saturating_add(2),
                vec![
                    Scene::text(base.saturating_add(3), title)
                        .with_style(Style::PLAIN.fg(Color::Ansi(6)).bold()),
                    header,
                    body,
                    footer,
                ],
            ),
        ),
    )
}

#[must_use]
pub fn labeled_value<Msg>(base: u64, label: &str, value: String, style: Style) -> Scene<Msg> {
    Scene::row(
        base,
        vec![
            Scene::text(base.saturating_add(1), format!("{label}: "))
                .with_style(Style::PLAIN.fg(Color::Ansi(8)).bold()),
            Scene::text(base.saturating_add(2), value).with_style(style),
        ],
    )
}

#[must_use]
pub fn wrap_text_lines(text: &str, width: u16) -> Vec<String> {
    let width = usize::from(width.max(8));
    let mut lines = Vec::new();

    for paragraph in text.split('\n') {
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            let next_len = if current.is_empty() {
                word.len()
            } else {
                current.len().saturating_add(1 + word.len())
            };

            if next_len > width && !current.is_empty() {
                lines.push(current);
                current = word.to_string();
            } else if current.is_empty() {
                current = word.to_string();
            } else {
                current.push(' ');
                current.push_str(word);
            }
        }

        if current.is_empty() {
            lines.push(String::new());
        } else {
            lines.push(current);
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

#[must_use]
pub fn truncated_wrapped_lines(text: &str, width: u16, max_lines: usize) -> Vec<String> {
    let mut lines = wrap_text_lines(text, width);
    if lines.len() > max_lines {
        lines.truncate(max_lines);
        if let Some(last) = lines.last_mut() {
            if last.len() >= 3 {
                last.truncate(last.len().saturating_sub(3));
            }
            last.push_str("...");
        }
    }
    lines
}
