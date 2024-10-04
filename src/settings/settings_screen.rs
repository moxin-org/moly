use makepad_code_editor::code_view::CodeViewWidgetExt;
use makepad_widgets::*;

use crate::data::{chats::model_loader::ModelLoaderStatus, store::Store};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import makepad_draw::shader::std::*;
    import makepad_code_editor::code_view::CodeView;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;

    BG_IMAGE = dep("crate://self/resources/images/my_models_bg_image.png")

    SettingsScreen = {{SettingsScreen}} {
        width: Fill
        height: Fill
        flow: Overlay

        <Image> {
            source: (BG_IMAGE),
            width: Fill,
            height: Fill,
        }

        <View> {
            width: Fill, height: Fill
            flow: Down
            align: {x: 0.0, y: 0.0}
            padding: 60

            spacing: 20

            header = <View> {
                width: Fill, height: Fit
                spacing: 15
                flow: Right
                align: {x: 0.0, y: 1.0}

                title = <Label> {
                    draw_text:{
                        text_style: <BOLD_FONT>{font_size: 30}
                        color: #000
                    }
                    text: "Settings"
                }
            }

            <HorizontalFiller> { height: 40 }

            <Label> {
                draw_text:{
                    text_style: <BOLD_FONT>{font_size: 16}
                    color: #000
                }
                text: "Local inference server information"
            }

            port_number_label = <Label> {
                draw_text:{
                    text_style: <REGULAR_FONT>{font_size: 12}
                    color: #000
                }
            }

            <HorizontalFiller> { height: 10 }

            <Label> {
                draw_text:{
                    text_style: <BOLD_FONT>{font_size: 16}
                    color: #000
                }
                text: "Client code examples"
            }

            code_snippet = <CodeView> {
                editor: {
                    pad_left_top: vec2(10.0,10.0)
                    width: Fill,
                    height: Fit,
                    draw_bg: { color: #3c3c3c },
                }
            }
        }
    }
}

#[derive(Widget, LiveHook, Live)]
pub struct SettingsScreen {
    #[deref]
    view: View,
}

impl Widget for SettingsScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get_mut::<Store>().unwrap();
        if let ModelLoaderStatus::Loaded(info) = store.chats.model_loader.status() {
            let port = info.listen_port;
            self.view.label(id!(port_number_label)).set_text(&format!("Assigned port number: {}", port));
            self.view.code_view(id!(code_snippet)).set_text(&format!("# Load a model and run this example in your terminal
# Choose between streaming and non-streaming mode by setting the \"stream\" field

curl http://localhost:{}/v1/chat/completions \\
-H \"Content-Type: application/json\" \\
-d '{{ 
  \"model\": \"moly-chat\",
  \"messages\": [ 
    {{ \"role\": \"system\", \"content\": \"Always answer in rhymes.\" }},
    {{ \"role\": \"user\", \"content\": \"Introduce yourself.\" }}
  ], 
  \"temperature\": 0.7, 
  \"max_tokens\": -1,
  \"stream\": true
}}'
                    ", port));
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for SettingsScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
    }
}
