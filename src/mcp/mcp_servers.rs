use crate::data::store::Store;
use crate::settings::sync_modal::SyncModalAction;
use makepad_code_editor::code_editor::{CodeEditorAction, KeepCursorInView};
use makepad_code_editor::decoration::DecorationSet;
use makepad_code_editor::{CodeDocument, CodeEditor, CodeSession};

use makepad_widgets::*;

use crate::data::mcp_servers::McpServersConfig;

live_design! {
    use link::widgets::*;
    use link::theme::*;
    use link::shaders::*;

    use crate::shared::widgets::*;
    use crate::shared::styles::*;
    use makepad_code_editor::code_editor::*;

    MolyCodeView = {{MolyCodeView}}{
        editor: <CodeEditor>{
            pad_left_top: vec2(0.0,-0.0)
            height:Fit
            empty_page_at_end: false,
            read_only: true,
            show_gutter: false
        }
    }

    McpCodeView = <MolyCodeView> {
        editor: {
            read_only: false
            margin: {top: -2, bottom: 2}
            pad_left_top: vec2(10.0,10.0)
            width: Fill,
            height: Fill,
            draw_bg: { color: #1d2330 },
            draw_text: {
                text_style: {
                    font_size: 9,
                }
            }

            // Inspired by Electron Highlighter theme https://electron-highlighter.github.io
            token_colors: {
                whitespace: #a8b5d1,        // General text/punctuation color as fallback
                delimiter: #a8b5d1,          // punctuation
                delimiter_highlight: #c5cee0, // Using a slightly brighter gray for highlight
                error_decoration: #f44747,   // token.error-token
                warning_decoration: #cd9731, // token.warn-token

                unknown: #a8b5d1,          // General text color
                branch_keyword: #d2a6ef,     // keyword.control
                constant: #ffd9af,         // constant.numeric
                identifier: #a8b5d1,         // variable
                loop_keyword: #d2a6ef,       // keyword.control.loop
                number: #ffd9af,           // constant.numeric
                other_keyword: #d2a6ef,      // keyword
                punctuator: #a8b5d1,         // punctuation
                string: #58ffc7,           // string
                function: #82aaff,         // entity.name.function
                typename: #fcf9c3,         // entity.name.class/type
                comment: #506686,          // comment
            }
        }
    }

    ServersEditorWrapper = <View> {
        <AdaptiveView> {
            Desktop = {
                mcp_code_view = <McpCodeView> {}
            }
            Mobile = {
                mcp_code_view = <MolyTextInput> {
                    width: Fill, height: Fill
                }
            }
        }
    }

    ServersEditor = <View> {
        width: Fill, height: Fill
        flow: Down
        padding: {left: 20, right: 20, bottom: 5}
        align: {x: 1.0}

        <View> {
            width: Fill, height: Fill
            padding: {left: 0, right: 25, top: 8, bottom: 8}
            servers_editor_wrapper = <ServersEditorWrapper> {}
        }

        <View> {
            width: Fill, height: Fit
            align: {x: 1.0, y: 0.5}
            padding: {left: 0, right: 15, top: 8, bottom: 8}

            save_button = <RoundedShadowView> {
                cursor: Hand
                margin: {left: 10, right: 10, bottom: 0, top: 0}
                width: Fit, height: Fit
                align: {x: 0.5, y: 0.5}
                padding: {left: 30, right: 30, bottom: 15, top: 15}
                draw_bg: {
                    color: (MAIN_BG_COLOR)
                    border_radius: 4.5,
                    uniform shadow_color: #0002
                    shadow_radius: 8.0,
                    shadow_offset: vec2(0.0,-1.5)
                }
                <Label> {
                    text: "Save and restart servers"
                    draw_text: {
                        text_style: <REGULAR_FONT>{font_size: 11}
                        color: #000
                    }
                }
            }
        }
    }

    SaveStatus = <View> {
        width: Fill, height: Fit,
        padding: {left: 10, right: 20, top: 8, bottom: 8}
        save_status = <Label> {
            draw_text: {
                text_style: <BOLD_FONT>{font_size: 10},
                color: #000
            }
        }
    }

    Instructions = <View> {
        // Ideally this should be width: Fill but for some reason it won't Fill
        width: 600, height: Fit,
        flow: Down, spacing: 10
        instructions = <Label> {
            width: Fill, height: Fit
            text: "Add new servers by editing the list under 'servers'. You can copy paste your\nconfiguration from other applications like Clade Desktop or VSCode."
            draw_text: {
                text_style: <REGULAR_FONT>{font_size: 11},
                color: #000
            }
        }
    }

    pub McpServers = {{McpServers}} {
        <AdaptiveView> {
            Desktop = {
                width: Fill, height: Fill
                flow: Right
                <ServersEditor> { width: 600 }
                <View> {
                    margin: {top: 10}
                    width: Fill, height: Fill
                    <Instructions> {}
                    <SaveStatus> {}
                }
            }
            Mobile = {
                <ScrollYView> {
                    flow: Down
                    padding: {left: 10}
                    <Instructions> {
                        padding: {left: 10}
                        <Label> {
                            text: "Note that only HTTP/SSE servers are supported on mobile devices"
                            draw_text: {
                                text_style: <BOLD_FONT>{font_size: 11}
                                color: #FFA000
                            }
                        }
                    }
                    <ServersEditor> { width: Fill }
                    <SaveStatus> {}
                }
            }
        }
    }
}

#[derive(Widget, Live)]
struct McpServers {
    #[deref]
    view: View,

    #[rust]
    mcp_servers_config: McpServersConfig,

    #[rust]
    initialized: bool,
}

impl LiveHook for McpServers {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        self.mcp_servers_config = McpServersConfig::create_sample();
    }
}

impl Widget for McpServers {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        let editor = self.widget(id!(mcp_code_view));

        if !self.initialized || editor.text().is_empty() {
            self.initialized = true;
            let store = scope.data.get::<Store>().unwrap();
            self.set_mcp_servers_config(cx, store.get_mcp_servers_config().clone());
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl McpServers {
    fn set_mcp_servers_config(&mut self, cx: &mut Cx, config: McpServersConfig) {
        self.mcp_servers_config = config;
        let display_json = self
            .mcp_servers_config
            .to_json()
            .unwrap_or_else(|_| "{}".to_string());

        self.widget(id!(mcp_code_view)).set_text(cx, &display_json);
    }
}

impl WidgetMatchEvent for McpServers {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        if self.view(id!(save_button)).finger_up(actions).is_some() {
            let json_text = self.widget(id!(mcp_code_view)).text();
            let store = scope.data.get_mut::<Store>().unwrap();

            match store.update_mcp_servers_from_json(&json_text) {
                Ok(()) => {
                    let config = McpServersConfig::from_json(&json_text).unwrap();
                    self.set_mcp_servers_config(cx, config);

                    self.label(id!(save_status)).set_text(cx, "");

                    self.redraw(cx);
                }
                Err(e) => {
                    self.label(id!(save_status)).set_text(cx, &format!("{}", e));
                    self.redraw(cx);
                }
            }
        }

        for action in actions {
            if let SyncModalAction::McpServersUpdated = action.cast() {
                let store = scope.data.get_mut::<Store>().unwrap();
                self.set_mcp_servers_config(cx, store.get_mcp_servers_config().clone());
                self.redraw(cx);
            }
        }
    }
}

/// Moly's version of Makepad's CodeView (broken upstream)
#[derive(Live, LiveHook, Widget)]
pub struct MolyCodeView {
    #[wrap]
    #[live]
    pub editor: CodeEditor,
    #[rust]
    pub session: Option<CodeSession>,
    #[live(false)]
    keep_cursor_at_end: bool,

    #[live]
    text: ArcStringMut,
}

impl MolyCodeView {
    pub fn lazy_init_session(&mut self) {
        if self.session.is_none() {
            let dec = DecorationSet::new();
            let doc = CodeDocument::new(self.text.as_ref().into(), dec);
            self.session = Some(CodeSession::new(doc));
            self.session.as_mut().unwrap().handle_changes();
            if self.keep_cursor_at_end {
                self.session.as_mut().unwrap().set_cursor_at_file_end();
                self.editor.keep_cursor_in_view = KeepCursorInView::Once
            }
        }
    }
}

impl Widget for MolyCodeView {
    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.lazy_init_session();
        let session = self.session.as_mut().unwrap();

        self.editor.draw_walk_editor(cx, session, walk);

        // TODO: Support text input for mobile devices
        // CodeEditor is not currently showing the IME on mobile devices (which also means keyboard input is ignored in simulators)
        // and showing it manually from outside causes some issues like duplicated text input.

        // Add IME support for mobile devices
        // if cx.has_key_focus(self.editor.area()) {
        //     // Get cursor position
        //     if let Some(last_selection_index) = session.last_added_selection_index() {
        //         let last_added_selection = &session.selections()[last_selection_index];
        //         let (cursor_x, cursor_y) = session.layout().logical_to_normalized_position(
        //             last_added_selection.cursor.position,
        //             last_added_selection.cursor.affinity,
        //         );

        //         let cursor_pos = dvec2(cursor_x, cursor_y);

        //         cx.show_text_ime(
        //             self.editor.area(),
        //             cursor_pos,
        //         );
        //     }
        // }

        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        self.lazy_init_session();
        let session = self.session.as_mut().unwrap();
        for action in self
            .editor
            .handle_event(cx, event, &mut Scope::empty(), session)
        {
            //cx.widget_action(uid, &scope.path, action);
            session.handle_changes();

            // Sync the text field back to match the document state for text changes
            match action {
                CodeEditorAction::TextDidChange => {
                    let document_text = session.document().as_text().to_string();
                    if self.text.as_ref() != &document_text {
                        self.text.as_mut_empty().clear();
                        self.text.as_mut_empty().push_str(&document_text);
                    }
                }
                _ => {}
            }
        }
    }

    fn text(&self) -> String {
        if let Some(session) = &self.session {
            session.document().as_text().to_string()
        } else {
            self.text.as_ref().to_string()
        }
    }

    fn set_text(&mut self, cx: &mut Cx, v: &str) {
        // Get current text to compare
        let current_text = if let Some(session) = &self.session {
            session.document().as_text().to_string()
        } else {
            self.text.as_ref().to_string()
        };

        if current_text != v {
            // Update the internal text field
            self.text.as_mut_empty().clear();
            self.text.as_mut_empty().push_str(v);

            // If we have an active session, replace the document content
            if let Some(session) = &mut self.session {
                session.document().replace(v.into());
                session.handle_changes();
            } else {
                // No session yet, will be created on next lazy_init_session
            }

            self.redraw(cx);
        }
    }
}
