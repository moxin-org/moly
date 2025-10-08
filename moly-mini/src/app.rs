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

        let nop = async { 0 };

        let future = async move {
            let mutex = std::sync::Mutex::new(0);
            let lock = mutex.lock().unwrap();
            let a = std::rc::Rc::new(0);
            let r = nop.await;
            println!("{}", lock);
            println!("{}", r);
            println!("{}", a);
        };

        //f(future);

        cx.spawner().spawn(future).unwrap();
    }
}

fn f(future: impl std::future::Future<Output = ()> + Send) {}

app_main!(App);
