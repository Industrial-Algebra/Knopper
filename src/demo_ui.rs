use crate::{Color, Padding, Scene, SizeConstraint, Style};

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
pub fn surface_panel<Msg>(
    base: u64,
    width: u16,
    title: &str,
    subtitle: &str,
    cues: &[PresenceCue],
    body: Scene<Msg>,
    active: bool,
) -> Scene<Msg> {
    Scene::border(
        base,
        Scene::padding(
            base.saturating_add(1),
            Padding::all(1),
            Scene::sized(
                base.saturating_add(2),
                SizeConstraint::width(width),
                Scene::column(
                    base.saturating_add(3),
                    vec![
                        Scene::text(base.saturating_add(4), title)
                            .with_style(section_title_style(active)),
                        Scene::text(base.saturating_add(5), subtitle)
                            .with_style(Style::PLAIN.fg(Color::Ansi(8))),
                        presence_strip(base.saturating_add(6), cues),
                        body,
                    ],
                ),
            ),
        ),
    )
    .with_style(surface_style(active))
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
