use crossterm::{
    cursor::MoveTo,
    event::{self, Event as CtEvent, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use knopper::{
    Key, KeyEvent, Rect, RenderOp, ReviewDemoContext, ReviewDemoMachine, ReviewDemoMsg, Runtime,
    RuntimeEvent,
};
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let initial_size = terminal::size().unwrap_or((80, 24));
    let mut bounds = Rect::new(0, 0, initial_size.0, initial_size.1);
    let mut ctx = ReviewDemoContext::for_bounds(bounds);
    let mut runtime = Runtime::new(ReviewDemoMachine::new(), ctx.clone(), ());
    runtime.dispatch(RuntimeEvent::Focus(ctx.tabs.tab_base_id));
    sync_runtime_meta(&mut runtime, bounds);

    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;

    let mut help_visible = true;
    loop {
        if refresh_bounds(&mut runtime, &mut ctx, &mut bounds) {
            runtime.invalidate_render_state();
        }
        sync_runtime_meta(&mut runtime, bounds);
        render_raw_frame(&mut stdout, &mut runtime, bounds, help_visible)?;

        match event::read()? {
            CtEvent::Key(key) if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) => {
                if key.code == KeyCode::Char('q') && !key.modifiers.contains(KeyModifiers::CONTROL)
                {
                    break;
                }
                if key.code == KeyCode::Char('?') {
                    help_visible = !help_visible;
                    continue;
                }
                if let Some(mapped) = map_crossterm_key(key.code, key.modifiers) {
                    handle_runtime_key(&mut runtime, &ctx, mapped);
                }
            }
            CtEvent::Resize(width, height) => {
                bounds = Rect::new(0, 0, width, height);
                ctx = ReviewDemoContext::for_bounds(bounds);
                runtime.set_context(ctx.clone());
                runtime.dispatch(RuntimeEvent::Resize(knopper::ResizeEvent { width, height }));
            }
            _ => {}
        }
    }

    execute!(stdout, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn sync_runtime_meta(runtime: &mut Runtime<ReviewDemoMachine>, bounds: Rect) {
    runtime.send(ReviewDemoMsg::FocusChanged(
        runtime.focus().current().cloned(),
    ));
    runtime.send(ReviewDemoMsg::CursorChanged(runtime.cursor(bounds)));
}

fn refresh_bounds(
    runtime: &mut Runtime<ReviewDemoMachine>,
    ctx: &mut ReviewDemoContext,
    bounds: &mut Rect,
) -> bool {
    let (width, height) = terminal::size().unwrap_or((bounds.width, bounds.height));
    if width == bounds.width && height == bounds.height {
        return false;
    }
    *bounds = Rect::new(0, 0, width, height);
    *ctx = ReviewDemoContext::for_bounds(*bounds);
    runtime.set_context(ctx.clone());
    runtime.dispatch(RuntimeEvent::Resize(knopper::ResizeEvent { width, height }));
    true
}

fn handle_runtime_key(
    runtime: &mut Runtime<ReviewDemoMachine>,
    ctx: &ReviewDemoContext,
    key: KeyEvent,
) {
    if let Some(msg) = ReviewDemoMachine::new().key_msg(runtime.focus(), &runtime.model(), key, ctx)
    {
        runtime.send(msg);
    } else {
        runtime.dispatch(RuntimeEvent::Key(key));
    }
}

fn map_crossterm_key(code: KeyCode, modifiers: KeyModifiers) -> Option<KeyEvent> {
    let ctrl = modifiers.contains(KeyModifiers::CONTROL);
    let alt = modifiers.contains(KeyModifiers::ALT);
    let shift = modifiers.contains(KeyModifiers::SHIFT);
    let key = match code {
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Enter => Key::Enter,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Tab => Key::Tab,
        KeyCode::BackTab => Key::Tab,
        KeyCode::Esc => Key::Escape,
        KeyCode::Char(ch) => Key::Char(ch),
        _ => return None,
    };
    Some(KeyEvent {
        key,
        ctrl,
        alt,
        shift: shift || matches!(code, KeyCode::BackTab),
    })
}

fn render_scene_buffer(runtime: &mut Runtime<ReviewDemoMachine>, bounds: Rect) -> Vec<String> {
    let width = usize::from(bounds.width.max(1));
    let height = usize::from(bounds.height.max(1));
    let mut rows = vec![vec![' '; width]; height];

    for op in runtime.render_ops(bounds) {
        match op {
            RenderOp::DrawText { rect, content, .. } => {
                let y = usize::from(rect.y);
                let x = usize::from(rect.x);
                if y >= rows.len() {
                    continue;
                }
                for (offset, ch) in content.chars().enumerate() {
                    let col = x.saturating_add(offset);
                    if col >= width {
                        break;
                    }
                    rows[y][col] = ch;
                }
            }
            RenderOp::DrawBorder { rect, .. } => draw_border(&mut rows, rect),
            RenderOp::Annotate { .. } | RenderOp::SetCursor { .. } => {}
        }
    }

    rows.into_iter()
        .map(|row| row.into_iter().collect::<String>().trim_end().to_string())
        .collect()
}

fn draw_border(rows: &mut [Vec<char>], rect: Rect) {
    let width = rows.first().map_or(0, Vec::len);
    if width == 0 || rows.is_empty() || rect.width == 0 || rect.height == 0 {
        return;
    }
    let left = usize::from(rect.x);
    let top = usize::from(rect.y);
    let right = left.saturating_add(usize::from(rect.width.saturating_sub(1)));
    let bottom = top.saturating_add(usize::from(rect.height.saturating_sub(1)));
    if top >= rows.len() || left >= width {
        return;
    }
    for x in left..=right.min(width.saturating_sub(1)) {
        if top < rows.len() {
            rows[top][x] = if x == left || x == right.min(width.saturating_sub(1)) {
                '+'
            } else {
                '-'
            };
        }
        if bottom < rows.len() {
            rows[bottom][x] = if x == left || x == right.min(width.saturating_sub(1)) {
                '+'
            } else {
                '-'
            };
        }
    }
    for y in top..=bottom.min(rows.len().saturating_sub(1)) {
        let clipped_right = right.min(width.saturating_sub(1));
        rows[y][left] = if y == top || y == bottom.min(rows.len().saturating_sub(1)) {
            '+'
        } else {
            '|'
        };
        rows[y][clipped_right] = if y == top || y == bottom.min(rows.len().saturating_sub(1)) {
            '+'
        } else {
            '|'
        };
    }
}

fn render_raw_frame(
    stdout: &mut io::Stdout,
    runtime: &mut Runtime<ReviewDemoMachine>,
    bounds: Rect,
    help_visible: bool,
) -> io::Result<()> {
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    let lines = render_scene_buffer(runtime, bounds);
    writeln!(
        stdout,
        "Knopper review demo  |  {}x{}",
        bounds.width, bounds.height
    )?;
    writeln!(stdout, "{}", "─".repeat(usize::from(bounds.width.max(1))))?;
    for line in lines
        .into_iter()
        .take(usize::from(bounds.height.saturating_sub(4)))
    {
        writeln!(stdout, "{line}")?;
    }
    writeln!(stdout, "{}", "─".repeat(usize::from(bounds.width.max(1))))?;
    if help_visible {
        writeln!(
            stdout,
            "q quit  |  tab / shift-tab focus  |  arrows navigate  |  enter commit  |  type to filter/draft"
        )?;
    }
    stdout.flush()
}
