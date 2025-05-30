use makepad_widgets::*;

live_design!(
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    use crate::ui::*;

    App = {{App}} {
        ui: <Ui> {}
    }
);

#[derive(Live, LiveHook)]
struct App {
    #[live]
    ui: WidgetRef,
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);
        moly_kit::live_design(cx);
        crate::meta::live_design(cx);
        crate::bot_selector::live_design(cx);
        crate::demo_chat::live_design(cx);
        crate::ui::live_design(cx);
    }
}

app_main!(App);
