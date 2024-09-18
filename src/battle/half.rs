use makepad_widgets::*;
use markdown::MarkdownAction;
use moxin_mae::{MaeAgent, MaeBackend};

use crate::data::{
    chats::{chat::ChatEntity, Chats},
    downloads::Downloads,
    search::Search,
    store::Store,
};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::chat::chat_panel::ChatPanel;

    Half = {{Half}} {
        <ChatPanel> {
            main = {
                main_prompt_input = {visible: false},
            }
        }
    }
}

#[derive(Live, Widget)]
pub struct Half {
    #[deref]
    view: View,

    #[rust]
    store: Option<Store>,
}

impl Widget for Half {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.store.is_none() {
            let store = scope.data.get::<Store>().unwrap();
            self.store = Some(build_sandboxed_store(store));
        }

        let mut store = self.store.take().unwrap();

        if let Event::Signal = event {
            store.process_event_signal();
        }

        let mut scope = Scope::with_data(&mut store);
        self.view.handle_event(cx, event, &mut scope);
        self.widget_match_event(cx, event, &mut scope);
        self.store = Some(store);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let mut store = self.store.take().unwrap();
        let mut scope = Scope::with_data(&mut store);
        while !self.view.draw_walk(cx, &mut scope, walk).is_done() {}
        self.store = Some(store);
        DrawStep::done()
    }
}

impl WidgetMatchEvent for Half {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {}
}

impl LiveHook for Half {}

impl Half {
    pub fn send_message(&mut self, message: String) {
        println!("Sending message: {}", message);
        let store = self.store.as_mut().unwrap();
        store
            .chats
            .get_current_chat()
            .unwrap()
            .borrow_mut()
            .send_message_to_agent(MaeAgent::Reasoner, message, &store.mae_backend);
    }
}

impl HalfRef {
    pub fn send_message(&self, message: String) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.send_message(message);
        }
    }
}

fn build_sandboxed_store(store: &Store) -> Store {
    let mut sandbox = Store {
        backend: store.backend.clone(),
        mae_backend: store.mae_backend.clone(),
        downloads: Downloads::new(store.backend.clone()),
        search: Search::new(store.backend.clone()),
        preferences: Default::default(),
        chats: Chats::new(store.backend.clone(), store.mae_backend.clone()),
    };
    sandbox.downloads.downloaded_files = store.downloads.downloaded_files.clone();
    // sandbox.chats.loaded_model = store.chats.loaded_model.clone();
    sandbox.chats.loaded_model = store
        .downloads
        .downloaded_files
        .first()
        .unwrap()
        .file
        .clone()
        .into();
    sandbox.chats.create_empty_chat();
    sandbox
        .chats
        .get_current_chat()
        .unwrap()
        .borrow_mut()
        .associated_entity = Some(ChatEntity::Agent(MaeAgent::Reasoner));

    sandbox
}
