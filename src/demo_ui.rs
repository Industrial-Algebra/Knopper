// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelTheme {
    pub frame: Style,
    pub body: Style,
    pub header: Style,
    pub title: Style,
    pub subtitle: Style,
    pub muted: Style,
}

#[must_use]
pub fn focus_style() -> Style {
    Style::PLAIN.fg(Color::Ansi(6)).bold()
}

#[must_use]
pub fn shell_theme() -> PanelTheme {
    PanelTheme {
        frame: Style::PLAIN.fg(Color::Ansi(8)).bg(Color::Ansi(0)),
        body: Style::PLAIN.bg(Color::Ansi(0)),
        header: Style::PLAIN.bg(Color::Ansi(0)),
        title: Style::PLAIN.fg(Color::Ansi(6)).bg(Color::Ansi(0)).bold(),
        subtitle: Style::PLAIN.fg(Color::Ansi(8)).bg(Color::Ansi(0)),
        muted: Style::PLAIN.fg(Color::Ansi(8)).bg(Color::Ansi(0)),
    }
}

#[must_use]
pub fn panel_theme(active: bool) -> PanelTheme {
    if active {
        PanelTheme {
            frame: Style::PLAIN.fg(Color::Ansi(6)).bg(Color::Ansi(0)).bold(),
            body: Style::PLAIN.bg(Color::Ansi(0)),
            header: Style::PLAIN.bg(Color::Ansi(0)),
            title: Style::PLAIN.fg(Color::Ansi(6)).bg(Color::Ansi(0)).bold(),
            subtitle: Style::PLAIN.fg(Color::Ansi(7)).bg(Color::Ansi(0)),
            muted: Style::PLAIN.fg(Color::Ansi(8)).bg(Color::Ansi(0)),
        }
    } else {
        PanelTheme {
            frame: Style::PLAIN.fg(Color::Ansi(8)).bg(Color::Ansi(0)),
            body: Style::PLAIN.bg(Color::Ansi(0)),
            header: Style::PLAIN.bg(Color::Ansi(0)),
            title: Style::PLAIN.fg(Color::Ansi(7)).bg(Color::Ansi(0)).bold(),
            subtitle: Style::PLAIN.fg(Color::Ansi(8)).bg(Color::Ansi(0)),
            muted: Style::PLAIN.fg(Color::Ansi(8)).bg(Color::Ansi(0)),
        }
    }
}

#[must_use]
pub fn reading_theme() -> PanelTheme {
    PanelTheme {
        frame: Style::PLAIN.fg(Color::Ansi(8)).bg(Color::Ansi(0)),
        body: Style::PLAIN.bg(Color::Ansi(0)),
        header: Style::PLAIN.bg(Color::Ansi(0)),
        title: Style::PLAIN.fg(Color::Ansi(7)).bg(Color::Ansi(0)).bold(),
        subtitle: Style::PLAIN.fg(Color::Ansi(8)).bg(Color::Ansi(0)),
        muted: Style::PLAIN.fg(Color::Ansi(8)).bg(Color::Ansi(0)),
    }
}

#[must_use]
pub fn status_theme() -> PanelTheme {
    PanelTheme {
        frame: Style::PLAIN.fg(Color::Ansi(8)).bg(Color::Ansi(0)),
        body: Style::PLAIN.bg(Color::Ansi(0)),
        header: Style::PLAIN.bg(Color::Ansi(0)),
        title: Style::PLAIN.fg(Color::Ansi(7)).bg(Color::Ansi(0)).bold(),
        subtitle: Style::PLAIN.fg(Color::Ansi(8)).bg(Color::Ansi(0)),
        muted: Style::PLAIN.fg(Color::Ansi(8)).bg(Color::Ansi(0)),
    }
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
pub fn presence_style(tone: PresenceTone) -> Style {
    match tone {
        PresenceTone::Local => focus_style(),
        PresenceTone::Collaborator => Style::PLAIN.fg(Color::Ansi(3)).bold(),
        PresenceTone::Passive => Style::PLAIN.fg(Color::Ansi(8)),
    }
}

#[must_use]
pub fn presence_strip<Msg>(base: u64, cues: &[PresenceCue]) -> Scene<Msg> {
    themed_presence_strip(base, cues, Style::PLAIN.fg(Color::Ansi(8)))
}

#[must_use]
fn themed_presence_strip<Msg>(base: u64, cues: &[PresenceCue], value_style: Style) -> Scene<Msg> {
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
                        .with_style(value_style),
                ];
                if index + 1 != cues.len() {
                    nodes.push(Scene::text(id.saturating_add(3), "  ").with_style(value_style));
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
    theme: &PanelTheme,
) -> Scene<Msg> {
    Scene::column(
        base,
        vec![
            Scene::text(base.saturating_add(1), truncate_text(title, width))
                .with_style(theme.title),
            Scene::text(base.saturating_add(2), truncate_text(subtitle, width))
                .with_style(theme.subtitle),
            Scene::sized(
                base.saturating_add(3),
                SizeConstraint::width(width),
                Scene::viewport(
                    base.saturating_add(4),
                    themed_presence_strip(base.saturating_add(5), cues, theme.muted),
                ),
            ),
        ],
    )
    .with_style(theme.header)
}

fn lattice_row(
    width: usize,
    row: usize,
    cell_width: usize,
    cell_height: usize,
    horizontal: char,
    vertical: char,
    intersection: char,
) -> String {
    let mut line = String::with_capacity(width);
    let horizontal_row = row.is_multiple_of(cell_height);
    for column in 0..width {
        let vertical_col = column.is_multiple_of(cell_width);
        let ch = match (horizontal_row, vertical_col) {
            (true, true) => intersection,
            (true, false) => horizontal,
            (false, true) => vertical,
            (false, false) => ' ',
        };
        line.push(ch);
    }
    line
}

fn panel_fill_row(fill: PanelFill, width: u16, row: usize) -> String {
    let width = usize::from(width.max(1));
    match fill {
        PanelFill::Plain => " ".repeat(width),
        PanelFill::Grid => lattice_row(width, row, 4, 2, '┈', '┊', '┼'),
        PanelFill::DenseGrid => lattice_row(width, row, 3, 2, '─', '│', '┼'),
        PanelFill::Bands => {
            let ch = if row.is_multiple_of(2) { '┈' } else { ' ' };
            std::iter::repeat_n(ch, width).collect()
        }
        PanelFill::Dots => {
            let mut line = String::with_capacity(width);
            for column in 0..width {
                let major_row = row.is_multiple_of(4);
                let major_col = column.is_multiple_of(8);
                let minor_row = row % 4 == 2;
                let minor_col = column % 8 == 4;
                line.push(if (major_row && major_col) || (minor_row && minor_col) {
                    '•'
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
        PanelFill::DenseGrid => Style::PLAIN.fg(Color::Ansi(7)).bg(Color::Ansi(0)),
        PanelFill::Bands => Style::PLAIN.fg(Color::Ansi(8)).bg(Color::Ansi(0)),
        PanelFill::Dots => Style::PLAIN.fg(Color::Ansi(8)).bg(Color::Ansi(0)),
    }
}

fn shell_fill_style(fill: PanelFill) -> Style {
    match fill {
        PanelFill::Plain => Style::PLAIN.bg(Color::Ansi(0)),
        _ => Style::PLAIN.fg(Color::Ansi(8)).bg(Color::Ansi(0)),
    }
}

fn shell_background<Msg>(base: u64, width: u16, height: u16, fill: PanelFill) -> Scene<Msg> {
    let width_usize = usize::from(width.max(1));
    let height_usize = usize::from(height.max(1));
    let gutter = (width_usize.min(30) / 3).max(6);
    Scene::column(
        base,
        (0..height_usize)
            .map(|offset| {
                let row = if offset < 3 || offset + 3 >= height_usize {
                    " ".repeat(width_usize)
                } else {
                    let source = panel_fill_row(fill, width, offset);
                    source
                        .chars()
                        .enumerate()
                        .map(|(index, ch)| {
                            let in_left_gutter = index < gutter;
                            let in_right_gutter = index + gutter >= width_usize;
                            if (in_left_gutter || in_right_gutter)
                                && matches!(ch, '┈' | '─' | '┊' | '│' | '┼')
                            {
                                ch
                            } else {
                                ' '
                            }
                        })
                        .collect()
                };
                Scene::text(
                    base.saturating_add(1 + u64::try_from(offset).unwrap_or(u64::MAX)),
                    row,
                )
                .with_style(shell_fill_style(fill))
            })
            .collect::<Vec<_>>(),
    )
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
    theme: PanelTheme,
) -> Scene<Msg> {
    panel_chrome_with_fill(base, width, height, header, body, theme, PanelFill::Plain)
}

#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn panel_chrome_with_fill<Msg>(
    base: u64,
    width: u16,
    height: u16,
    header: Scene<Msg>,
    body: Scene<Msg>,
    theme: PanelTheme,
    fill: PanelFill,
) -> Scene<Msg> {
    let body_height = height.saturating_sub(PANEL_HEADER_ROWS).max(1);
    let body = body.with_style(theme.body);
    let body = if fill == PanelFill::Plain {
        body
    } else {
        Scene::stack(
            base.saturating_add(6),
            vec![
                panel_background(base.saturating_add(7), width, body_height, fill),
                body,
            ],
        )
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
                        header.with_style(theme.header),
                        Scene::sized(
                            base.saturating_add(4),
                            SizeConstraint::new(Some(width), Some(body_height)),
                            Scene::viewport(base.saturating_add(5), body),
                        ),
                    ],
                )
                .with_style(theme.body),
            ),
        ),
    )
    .with_style(theme.frame)
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
    surface_panel_with_theme(
        base,
        width,
        title,
        subtitle,
        cues,
        body,
        panel_theme(active),
    )
}

#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn surface_panel_with_theme<Msg>(
    base: u64,
    width: u16,
    title: &str,
    subtitle: &str,
    cues: &[PresenceCue],
    body: Scene<Msg>,
    theme: PanelTheme,
) -> Scene<Msg> {
    bounded_surface_panel_with_theme(
        base,
        width,
        PANEL_HEADER_ROWS.saturating_add(1).saturating_add(8),
        title,
        subtitle,
        cues,
        body,
        theme,
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
    bounded_surface_panel_with_theme(
        base,
        width,
        height,
        title,
        subtitle,
        cues,
        body,
        panel_theme(active),
    )
}

#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn bounded_surface_panel_with_theme<Msg>(
    base: u64,
    width: u16,
    height: u16,
    title: &str,
    subtitle: &str,
    cues: &[PresenceCue],
    body: Scene<Msg>,
    theme: PanelTheme,
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
            &theme,
        ),
        body,
        theme,
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
    let theme = panel_theme(active);
    panel_chrome_with_fill(
        base,
        width,
        height,
        panel_header(
            base.saturating_add(20),
            width,
            title,
            subtitle,
            cues,
            &theme,
        ),
        body,
        theme,
        if active {
            active_panel_fill(fill)
        } else {
            fill
        },
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
    app_shell_with_theme(
        base,
        width,
        height,
        title,
        header,
        body,
        footer,
        shell_theme(),
    )
}

#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn app_shell_with_theme<Msg>(
    base: u64,
    width: u16,
    height: u16,
    title: &str,
    header: Scene<Msg>,
    body: Scene<Msg>,
    footer: Scene<Msg>,
    theme: PanelTheme,
) -> Scene<Msg> {
    Scene::border(
        base,
        Scene::sized(
            base.saturating_add(1),
            SizeConstraint::new(Some(width), Some(height)),
            Scene::column(
                base.saturating_add(2),
                vec![
                    Scene::text(base.saturating_add(3), title).with_style(theme.title),
                    header.with_style(theme.header),
                    body.with_style(theme.body),
                    footer.with_style(theme.body),
                ],
            )
            .with_style(theme.body),
        ),
    )
    .with_style(theme.frame)
}

#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn app_shell_with_fill<Msg>(
    base: u64,
    width: u16,
    height: u16,
    title: &str,
    header: Scene<Msg>,
    body: Scene<Msg>,
    footer: Scene<Msg>,
    fill: PanelFill,
) -> Scene<Msg> {
    app_shell_with_theme_and_fill(
        base,
        width,
        height,
        title,
        header,
        body,
        footer,
        shell_theme(),
        fill,
    )
}

#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn app_shell_with_theme_and_fill<Msg>(
    base: u64,
    width: u16,
    height: u16,
    title: &str,
    header: Scene<Msg>,
    body: Scene<Msg>,
    footer: Scene<Msg>,
    theme: PanelTheme,
    fill: PanelFill,
) -> Scene<Msg> {
    if fill == PanelFill::Plain {
        return app_shell_with_theme(base, width, height, title, header, body, footer, theme);
    }

    Scene::border(
        base,
        Scene::sized(
            base.saturating_add(1),
            SizeConstraint::new(Some(width), Some(height)),
            Scene::stack(
                base.saturating_add(2),
                vec![
                    shell_background(base.saturating_add(3), width, height, fill),
                    Scene::column(
                        base.saturating_add(4),
                        vec![
                            Scene::text(base.saturating_add(5), title).with_style(theme.title),
                            header.with_style(theme.header),
                            body.with_style(theme.body),
                            footer.with_style(theme.body),
                        ],
                    )
                    .with_style(theme.body),
                ],
            ),
        ),
    )
    .with_style(theme.frame)
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
