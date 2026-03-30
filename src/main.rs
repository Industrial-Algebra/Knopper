use knopper::{
    ButtonMsg, DemoContext, DemoMachine, DemoMsg, Rect, RenderOp, Runtime, TabsMsg, TextareaMsg,
    ToggleMsg,
};

fn main() {
    let mut runtime = Runtime::new(DemoMachine::new(), DemoContext::default(), ());

    runtime.send(DemoMsg::Toggle(ToggleMsg::Toggle));
    runtime.send(DemoMsg::Button(ButtonMsg::Press));
    runtime.send(DemoMsg::Tabs(TabsMsg::Select(1)));
    runtime.send(DemoMsg::Textarea(TextareaMsg::Insert('K')));
    runtime.send(DemoMsg::Textarea(TextareaMsg::Insert('n')));
    runtime.send(DemoMsg::Textarea(TextareaMsg::Insert('o')));
    runtime.send(DemoMsg::Textarea(TextareaMsg::Insert('p')));
    runtime.send(DemoMsg::Textarea(TextareaMsg::Insert('p')));
    runtime.send(DemoMsg::Textarea(TextareaMsg::Insert('e')));
    runtime.send(DemoMsg::Textarea(TextareaMsg::Insert('r')));
    runtime.send(DemoMsg::Textarea(TextareaMsg::Newline));
    runtime.send(DemoMsg::Textarea(TextareaMsg::Insert('d')));
    runtime.send(DemoMsg::Textarea(TextareaMsg::Insert('e')));
    runtime.send(DemoMsg::Textarea(TextareaMsg::Insert('m')));
    runtime.send(DemoMsg::Textarea(TextareaMsg::Insert('o')));
    runtime.send(DemoMsg::Textarea(TextareaMsg::Commit));

    let bounds = Rect::new(0, 0, 80, 24);
    let mut text_ops: Vec<_> = runtime
        .render_ops(bounds)
        .into_iter()
        .filter_map(|op| match op {
            RenderOp::DrawText { rect, content, .. } => Some((rect.y, rect.x, content)),
            _ => None,
        })
        .collect();
    text_ops.sort_by_key(|(y, x, _)| (*y, *x));

    println!("Knopper demo workspace snapshot\n");
    for (y, x, content) in text_ops {
        println!("({x:02},{y:02}) {content}");
    }

    if let Some((x, y)) = runtime.cursor(bounds) {
        println!("\nCursor: ({x},{y})");
    }

    println!("\nThis binary currently prints a rendered snapshot of the demo scene.");
    println!("See docs/roadmap/00-first-release-roadmap.md for the release plan.");
}
