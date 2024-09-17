use std::borrow::BorrowMut;

use makepad_widgets::*;
use markdown::MarkdownAction;
use moxin_mae::{MaeAgent, MaeBackend};

use crate::data::{
    chats::{chat::ChatEntity, Chats},
    downloads::Downloads,
    search::Search,
    store::Store,
};

use super::{messages::MessagesWidgetExt, prompt::PromptWidgetExt};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::battle::messages::Messages;
    import crate::battle::prompt::Prompt;
    import crate::battle::agent_selector::AgentSelector;
    import crate::chat::chat_panel::ChatPanel;

    GAP = 12;

    Half = <ChatPanel> {}

    BattleScreen = {{BattleScreen}} {
        content = <View> {
            flow: Down,
            visible: false,
            padding: (GAP),
            spacing: (GAP),
            <View> {
                spacing: (GAP),
                left = <Half> {}
                right = <Half> {}
            }
            // prompt = <Prompt> {}
        }
    }
}

#[derive(Live, Widget)]
pub struct BattleScreen {
    #[deref]
    view: View,

    #[rust]
    left_store: Option<Store>,

    #[rust]
    right_store: Option<Store>,
}

impl Widget for BattleScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.left_store.is_none() {
            let store = scope.data.get::<Store>().unwrap();
            self.left_store = Some(build_sandboxed_store(store));
            self.right_store = Some(build_sandboxed_store(store));
        }

        if let Event::Signal = event {
            self.left_store.as_mut().unwrap().process_event_signal();
            self.right_store.as_mut().unwrap().process_event_signal();
        }

        let mut store = self.left_store.take().unwrap();
        let mut scope = Scope::with_data(&mut store);
        self.view.handle_event(cx, event, &mut scope);
        self.widget_match_event(cx, event, &mut scope);
        self.left_store = Some(store);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let mut store = self.left_store.take().unwrap();
        let mut scope = Scope::with_data(&mut store);
        while !self.view.draw_walk(cx, &mut scope, walk).is_done() {}
        self.left_store = Some(store);
        DrawStep::done()
    }
}

impl WidgetMatchEvent for BattleScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        for action in actions {
            if let MarkdownAction::LinkNavigated(url) = action.as_widget_action().cast() {
                let _ = robius_open::Uri::new(&url).open();
            }
        }
    }
}

impl LiveHook for BattleScreen {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        // Enable this screen only if there are enough agents, quick solution.
        if MaeBackend::available_agents().len() >= 2 {
            self.view(id!(content)).set_visible(true);
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
