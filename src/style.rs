#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Default,
    Ansi(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Emphasis {
    pub bold: bool,
    pub italic: bool,
    pub underlined: bool,
    pub reversed: bool,
}

impl Emphasis {
    #[must_use]
    pub fn combine(self, other: Self) -> Self {
        Self {
            bold: self.bold || other.bold,
            italic: self.italic || other.italic,
            underlined: self.underlined || other.underlined,
            reversed: self.reversed || other.reversed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub emphasis: Emphasis,
}

impl Style {
    pub const PLAIN: Self = Self {
        fg: None,
        bg: None,
        emphasis: Emphasis {
            bold: false,
            italic: false,
            underlined: false,
            reversed: false,
        },
    };

    #[must_use]
    pub fn combine(self, other: Self) -> Self {
        Self {
            fg: other.fg.or(self.fg),
            bg: other.bg.or(self.bg),
            emphasis: self.emphasis.combine(other.emphasis),
        }
    }

    #[must_use]
    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    #[must_use]
    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    #[must_use]
    pub fn bold(mut self) -> Self {
        self.emphasis.bold = true;
        self
    }

    #[must_use]
    pub fn underlined(mut self) -> Self {
        self.emphasis.underlined = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_combine_prefers_more_specific_colors_and_accumulates_emphasis() {
        let base = Style::PLAIN.fg(Color::Ansi(2)).bold();
        let overlay = Style::PLAIN.bg(Color::Ansi(4)).underlined();

        let combined = base.combine(overlay);

        assert_eq!(combined.fg, Some(Color::Ansi(2)));
        assert_eq!(combined.bg, Some(Color::Ansi(4)));
        assert!(combined.emphasis.bold);
        assert!(combined.emphasis.underlined);
    }
}
