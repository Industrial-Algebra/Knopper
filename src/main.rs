use crossterm::{
    cursor::MoveTo,
    event::{self, Event as CtEvent, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use knopper::{DemoContext, DemoMachine, Key, KeyEvent, Rect, RenderOp, Runtime, RuntimeEvent};
use std::io::{self, Write};

#[cfg(feature = "notcurses")]
use knopper::NotcursesBackend;

#[cfg(feature = "notcurses")]
type DemoBackend<'a> = Option<&'a mut NotcursesBackend>;
#[cfg(not(feature = "notcurses"))]
type DemoBackend<'a> = Option<&'a mut ()>;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let use_notcurses = args.iter().any(|arg| arg == "--notcurses");
    let use_shell = args.iter().any(|arg| arg == "--shell");

    let initial_size = terminal::size().unwrap_or((80, 24));
    let bounds = Rect::new(0, 0, initial_size.0, initial_size.1);
    let ctx = DemoContext::for_bounds(bounds);
    let mut runtime = Runtime::new(DemoMachine::new(), ctx.clone(), ());
    runtime.dispatch(RuntimeEvent::Focus(ctx.tabs.tab_base_id));
    sync_demo_runtime_meta(&mut runtime, bounds);

    #[cfg(feature = "notcurses")]
    let mut backend = if use_notcurses {
        match NotcursesBackend::new() {
            Ok(backend) => Some(backend),
            Err(error) => {
                eprintln!("failed to start Notcurses backend: {error}");
                None
            }
        }
    } else {
        None
    };

    #[cfg(not(feature = "notcurses"))]
    if use_notcurses {
        eprintln!("binary was not built with the `notcurses` feature; using stdout rendering only");
    }

    if use_shell {
        print_shell_help();
        render_demo(&mut runtime, bounds);
        #[cfg(feature = "notcurses")]
        render_notcurses(&mut runtime, bounds, backend.as_mut());
        run_shell(&ctx, bounds, &mut runtime, {
            #[cfg(feature = "notcurses")]
            {
                backend.as_mut()
            }
            #[cfg(not(feature = "notcurses"))]
            {
                None
            }
        });
    } else {
        let result = run_raw_host(ctx.clone(), bounds, &mut runtime, {
            #[cfg(feature = "notcurses")]
            {
                backend.as_mut()
            }
            #[cfg(not(feature = "notcurses"))]
            {
                None
            }
        });
        if let Err(error) = result {
            eprintln!("interactive host failed: {error}");
        }
    }
}

#[cfg_attr(not(feature = "notcurses"), allow(unused_mut))]
fn run_shell(
    ctx: &DemoContext,
    bounds: Rect,
    runtime: &mut Runtime<DemoMachine>,
    #[allow(unused_variables)] mut backend: DemoBackend<'_>,
) {
    let stdin = io::stdin();
    loop {
        print!("\nknopper-demo> ");
        let _ = io::stdout().flush();

        let mut line = String::new();
        if stdin.read_line(&mut line).is_err() {
            eprintln!("failed to read command");
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match handle_command(line, runtime, ctx) {
            CommandResult::Continue => {
                sync_demo_runtime_meta(runtime, bounds);
                render_demo(runtime, bounds);
                #[cfg(feature = "notcurses")]
                if let Some(backend) = backend.as_deref_mut() {
                    render_notcurses(runtime, bounds, Some(backend));
                }
            }
            CommandResult::Quit => break,
            CommandResult::Help => print_shell_help(),
            CommandResult::Error(message) => eprintln!("{message}"),
        }
    }
}

#[cfg_attr(not(feature = "notcurses"), allow(unused_mut))]
fn run_raw_host(
    mut ctx: DemoContext,
    mut bounds: Rect,
    runtime: &mut Runtime<DemoMachine>,
    #[allow(unused_variables)] mut backend: DemoBackend<'_>,
) -> io::Result<()> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;

    let mut help_visible = true;
    loop {
        refresh_raw_host_bounds(runtime, &mut ctx, &mut bounds);
        sync_demo_runtime_meta(runtime, bounds);
        render_raw_frame(&mut stdout, runtime, bounds, help_visible)?;
        #[cfg(feature = "notcurses")]
        if let Some(backend) = backend.as_deref_mut() {
            render_notcurses(runtime, bounds, Some(backend));
        }

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
                if key.code == KeyCode::Esc && !runtime.model().palette.open {
                    break;
                }
                refresh_raw_host_bounds(runtime, &mut ctx, &mut bounds);
                if let Some(mapped) = map_crossterm_key(key.code, key.modifiers) {
                    handle_runtime_key(runtime, &ctx, mapped);
                }
            }
            CtEvent::Resize(width, height) => {
                bounds = Rect::new(0, 0, width, height);
                ctx = DemoContext::for_bounds(bounds);
                runtime.set_context(ctx.clone());
                runtime.dispatch(RuntimeEvent::Resize(knopper::ResizeEvent { width, height }));
                sync_demo_runtime_meta(runtime, bounds);
            }
            _ => {}
        }
    }

    execute!(stdout, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn refresh_raw_host_bounds(
    runtime: &mut Runtime<DemoMachine>,
    ctx: &mut DemoContext,
    bounds: &mut Rect,
) {
    if let Ok((width, height)) = terminal::size() {
        let next = Rect::new(0, 0, width, height);
        if *bounds != next {
            *bounds = next;
            *ctx = DemoContext::for_bounds(next);
            runtime.set_context(ctx.clone());
            runtime.dispatch(RuntimeEvent::Resize(knopper::ResizeEvent { width, height }));
        }
    }
}

fn sync_demo_runtime_meta(runtime: &mut Runtime<DemoMachine>, bounds: Rect) {
    let focus = runtime.focus().current().cloned();
    let cursor = runtime.cursor(bounds);
    if runtime.model().focused != focus {
        runtime.send(knopper::DemoMsg::FocusChanged(focus));
    }
    if runtime.model().cursor != cursor {
        runtime.send(knopper::DemoMsg::CursorChanged(cursor));
    }
}

fn handle_runtime_key(runtime: &mut Runtime<DemoMachine>, ctx: &DemoContext, key: KeyEvent) {
    if matches!(key.key, Key::Tab) {
        runtime.dispatch(RuntimeEvent::Key(key));
        return;
    }

    let machine = DemoMachine::new();
    let model = runtime.model();
    let focus = runtime.focus().clone();
    if let Some(msg) = machine.key_msg(&focus, &model, key, ctx) {
        runtime.send(msg);
    } else {
        runtime.dispatch(RuntimeEvent::Key(key));
    }
}

fn map_crossterm_key(code: KeyCode, modifiers: KeyModifiers) -> Option<KeyEvent> {
    let key = match code {
        KeyCode::Char(ch) => Key::Char(ch),
        KeyCode::Enter => Key::Enter,
        KeyCode::Esc => Key::Escape,
        KeyCode::Tab | KeyCode::BackTab => Key::Tab,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        _ => return None,
    };

    Some(KeyEvent {
        key,
        ctrl: modifiers.contains(KeyModifiers::CONTROL),
        alt: modifiers.contains(KeyModifiers::ALT),
        shift: modifiers.contains(KeyModifiers::SHIFT) || matches!(code, KeyCode::BackTab),
    })
}

enum CommandResult {
    Continue,
    Quit,
    Help,
    Error(String),
}

fn handle_command(
    line: &str,
    runtime: &mut Runtime<DemoMachine>,
    ctx: &DemoContext,
) -> CommandResult {
    if matches!(line, "quit" | "exit") {
        return CommandResult::Quit;
    }
    if matches!(line, "help" | "?") {
        return CommandResult::Help;
    }
    if line == "show" {
        return CommandResult::Continue;
    }

    let mut parts = line.split_whitespace();
    let Some(command) = parts.next() else {
        return CommandResult::Continue;
    };

    match command {
        "focus" => match parts.next() {
            Some("tabs") => runtime.dispatch(RuntimeEvent::Focus(ctx.tabs.tab_base_id)),
            Some("toggle") => runtime.dispatch(RuntimeEvent::Focus(ctx.toggle.label_id)),
            Some("button") => runtime.dispatch(RuntimeEvent::Focus(ctx.button.label_id)),
            Some("note") => runtime.dispatch(RuntimeEvent::Focus(ctx.textarea.line_base_id)),
            Some("list") => runtime.dispatch(RuntimeEvent::Focus(ctx.list.marker_base())),
            Some("palette") => runtime.dispatch(RuntimeEvent::Focus(ctx.palette.input.root_id)),
            Some(other) => return CommandResult::Error(format!("unknown focus target: {other}")),
            None => {
                return CommandResult::Error(
                    "usage: focus tabs|toggle|button|note|list|palette".into(),
                );
            }
        },
        "tab" => runtime.dispatch(RuntimeEvent::Key(KeyEvent {
            key: Key::Tab,
            ctrl: false,
            alt: false,
            shift: false,
        })),
        "backtab" => runtime.dispatch(RuntimeEvent::Key(KeyEvent {
            key: Key::Tab,
            ctrl: false,
            alt: false,
            shift: true,
        })),
        "tabs" => match parts.next() {
            Some("left") => runtime.send(knopper::DemoMsg::Tabs(knopper::TabsMsg::MoveLeft)),
            Some("right") => runtime.send(knopper::DemoMsg::Tabs(knopper::TabsMsg::MoveRight)),
            Some("commit") => {
                let index = runtime.model().tabs.selected;
                runtime.send(knopper::DemoMsg::Tabs(knopper::TabsMsg::Commit(index)));
            }
            Some("select") => match parts.next().and_then(|index| index.parse::<usize>().ok()) {
                Some(index) => runtime.send(knopper::DemoMsg::Tabs(knopper::TabsMsg::Select(index))),
                None => return CommandResult::Error("usage: tabs select <index>".into()),
            },
            Some(other) => return CommandResult::Error(format!("unknown tabs command: {other}")),
            None => {
                return CommandResult::Error("usage: tabs left|right|commit|select <index>".into());
            }
        },
        "toggle" => runtime.send(knopper::DemoMsg::Toggle(knopper::ToggleMsg::Toggle)),
        "sync" => runtime.send(knopper::DemoMsg::Button(knopper::ButtonMsg::Press)),
        "note" => match parts.next() {
            Some("text") => {
                let prefix = "note text";
                let text = line
                    .strip_prefix(prefix)
                    .map(str::trim_start)
                    .unwrap_or_default();
                for ch in text.chars() {
                    runtime.send(knopper::DemoMsg::Textarea(knopper::TextareaMsg::Insert(ch)));
                }
            }
            Some("nl") => runtime.send(knopper::DemoMsg::Textarea(knopper::TextareaMsg::Newline)),
            Some("backspace") => runtime.send(knopper::DemoMsg::Textarea(knopper::TextareaMsg::Backspace)),
            Some("left") => runtime.send(knopper::DemoMsg::Textarea(knopper::TextareaMsg::MoveLeft)),
            Some("right") => runtime.send(knopper::DemoMsg::Textarea(knopper::TextareaMsg::MoveRight)),
            Some("up") => runtime.send(knopper::DemoMsg::Textarea(knopper::TextareaMsg::MoveUp)),
            Some("down") => runtime.send(knopper::DemoMsg::Textarea(knopper::TextareaMsg::MoveDown)),
            Some("commit") => runtime.send(knopper::DemoMsg::Textarea(knopper::TextareaMsg::Commit)),
            Some(other) => return CommandResult::Error(format!("unknown note command: {other}")),
            None => {
                return CommandResult::Error(
                    "usage: note text <text>|nl|backspace|left|right|up|down|commit".into(),
                );
            }
        },
        "list" => match parts.next() {
            Some("up") => runtime.send(knopper::DemoMsg::List(knopper::ListMsg::MoveUp)),
            Some("down") => runtime.send(knopper::DemoMsg::List(knopper::ListMsg::MoveDown)),
            Some("commit") => {
                let index = runtime.model().list.selected;
                runtime.send(knopper::DemoMsg::List(knopper::ListMsg::Commit(index)));
            }
            Some(other) => return CommandResult::Error(format!("unknown list command: {other}")),
            None => return CommandResult::Error("usage: list up|down|commit".into()),
        },
        "palette" => match parts.next() {
            Some("open") => runtime.send(knopper::DemoMsg::OpenPalette),
            Some("close") => runtime.send(knopper::DemoMsg::Palette(knopper::CommandPaletteMsg::Dismiss)),
            Some("focus-input") => runtime.send(knopper::DemoMsg::Palette(knopper::CommandPaletteMsg::FocusInput)),
            Some("focus-list") => runtime.send(knopper::DemoMsg::Palette(knopper::CommandPaletteMsg::FocusList)),
            Some("up") => runtime.send(knopper::DemoMsg::Palette(knopper::CommandPaletteMsg::List(knopper::ListMsg::MoveUp))),
            Some("down") => runtime.send(knopper::DemoMsg::Palette(knopper::CommandPaletteMsg::List(knopper::ListMsg::MoveDown))),
            Some("commit") => {
                let index = runtime.model().palette.list.selected;
                runtime.send(knopper::DemoMsg::Palette(knopper::CommandPaletteMsg::List(knopper::ListMsg::Commit(index))));
            }
            Some("text") => {
                let prefix = "palette text";
                let text = line
                    .strip_prefix(prefix)
                    .map(str::trim_start)
                    .unwrap_or_default();
                for ch in text.chars() {
                    runtime.send(knopper::DemoMsg::Palette(knopper::CommandPaletteMsg::Input(knopper::InputMsg::Insert(ch))));
                }
            }
            Some("backspace") => runtime.send(knopper::DemoMsg::Palette(knopper::CommandPaletteMsg::Input(knopper::InputMsg::Backspace))),
            Some("tab") => runtime.send(knopper::DemoMsg::Palette(knopper::CommandPaletteMsg::FocusList)),
            Some("backtab") => runtime.send(knopper::DemoMsg::Palette(knopper::CommandPaletteMsg::FocusInput)),
            Some(other) => return CommandResult::Error(format!("unknown palette command: {other}")),
            None => return CommandResult::Error("usage: palette open|close|focus-input|focus-list|text <text>|backspace|up|down|commit|tab|backtab".into()),
        },
        other => return CommandResult::Error(format!("unknown command: {other}")),
    }

    CommandResult::Continue
}

fn render_demo(runtime: &mut Runtime<DemoMachine>, bounds: Rect) {
    println!("{}", snapshot_text(runtime, bounds));
}

fn snapshot_text(runtime: &mut Runtime<DemoMachine>, bounds: Rect) -> String {
    let mut text_ops: Vec<_> = runtime
        .render_ops(bounds)
        .into_iter()
        .filter_map(|op| match op {
            RenderOp::DrawText { rect, content, .. } => Some((rect.y, rect.x, content)),
            _ => None,
        })
        .collect();
    text_ops.sort_by_key(|(y, x, _)| (*y, *x));

    let mut out = String::from("Knopper demo workspace snapshot\n\n");
    for (y, x, content) in text_ops {
        out.push_str(&format!("({x:02},{y:02}) {content}\n"));
    }
    out.push_str(&format!("\nStatus model: {:?}\n", runtime.model()));
    out.push_str(&format!("Focus: {:?}\n", runtime.focus().current()));
    out.push_str(&format!("Cursor: {:?}\n", runtime.cursor(bounds)));
    out
}

fn render_scene_buffer(runtime: &mut Runtime<DemoMachine>, bounds: Rect) -> Vec<String> {
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
        if left < width {
            rows[y][left] = if y == top || y == bottom.min(rows.len().saturating_sub(1)) {
                '+'
            } else {
                '|'
            };
        }
        let clipped_right = right.min(width.saturating_sub(1));
        if clipped_right < width {
            rows[y][clipped_right] = if y == top || y == bottom.min(rows.len().saturating_sub(1)) {
                '+'
            } else {
                '|'
            };
        }
    }
}

fn render_raw_frame(
    stdout: &mut io::Stdout,
    runtime: &mut Runtime<DemoMachine>,
    bounds: Rect,
    help_visible: bool,
) -> io::Result<()> {
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

    let model = runtime.model();
    let lines = render_scene_buffer(runtime, bounds);
    let active_tab = ["Overview", "Notes", "Tasks"]
        .get(model.tabs.selected)
        .copied()
        .unwrap_or("<none>");
    let note = model
        .textarea
        .committed
        .as_deref()
        .unwrap_or("<draft>")
        .replace('\n', " | ");
    let focus = runtime
        .focus()
        .current()
        .map(|path| format!("{:?}", path.current()))
        .unwrap_or_else(|| "none".into());

    let rule_width = usize::from(bounds.width.max(1));
    writeln!(
        stdout,
        "Knopper demo  |  {}x{}  |  tab:{active_tab}  shared:{}  syncs:{}  palette:{}",
        bounds.width,
        bounds.height,
        model.toggle.checked,
        model.button.activations,
        model.palette.open
    )?;
    writeln!(stdout, "{}", "─".repeat(rule_width))?;
    for line in lines
        .into_iter()
        .take(usize::from(bounds.height.saturating_sub(5)))
    {
        writeln!(stdout, "{line}")?;
    }
    writeln!(stdout, "{}", "─".repeat(rule_width))?;
    writeln!(stdout, "note: {note}")?;
    writeln!(
        stdout,
        "focus: {focus}  cursor: {:?}",
        runtime.cursor(bounds)
    )?;
    if help_visible {
        writeln!(
            stdout,
            "keys: q quit  ? help  Ctrl-P palette  Tab / Shift-Tab focus  arrows move  Enter commit/activate  Esc quit/close"
        )?;
    } else {
        writeln!(stdout, "press ? for key help")?;
    }
    stdout.flush()
}

#[cfg(feature = "notcurses")]
fn render_notcurses(
    runtime: &mut Runtime<DemoMachine>,
    bounds: Rect,
    backend: Option<&mut NotcursesBackend>,
) {
    if let Some(backend) = backend
        && let Err(error) = runtime.render_to_backend_auto_cursor(backend, bounds)
    {
        eprintln!("failed to render to backend: {error}");
    }
}

#[cfg(not(feature = "notcurses"))]
#[allow(dead_code)]
fn render_notcurses(_runtime: &mut Runtime<DemoMachine>, _bounds: Rect, _backend: DemoBackend<'_>) {
}

fn print_shell_help() {
    println!(
        "Commands:\n  show\n  help\n  quit\n  focus tabs|toggle|button|note|list|palette\n  tab\n  backtab\n  tabs left|right|commit|select <index>\n  toggle\n  sync\n  note text <text>\n  note nl\n  note backspace\n  note left|right|up|down\n  note commit\n  list up|down|commit\n  palette open|close\n  palette focus-input|focus-list\n  palette text <text>\n  palette backspace\n  palette up|down|commit\n  palette tab|backtab\n\nDefault run mode is raw-key interactive host. Use --shell for the old command shell. Use `cargo run --features notcurses -- --notcurses` to also render each frame through the Notcurses backend."
    );
}

trait DemoListFocusExt {
    fn marker_base(&self) -> knopper::NodeId;
}

impl DemoListFocusExt for knopper::ListContext<String> {
    fn marker_base(&self) -> knopper::NodeId {
        knopper::ListIds::default().marker_base
    }
}
