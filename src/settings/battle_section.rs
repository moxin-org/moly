use makepad_widgets::*;

use crate::data::store::ScopeStoreExt;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;

    BattleSection = {{BattleSection}} {
        flow: Down,
        height: Fit,
        <Label> {
            text: "Arena",
            draw_text: {
                text_style: <BOLD_FONT>{font_size: 16}
                color: #000
            }
        }
        <View> {
            height: Fit,
            align: {y: 0.5}
            spacing: 4,
            <Label> {
                text: "Server:",
                draw_text: {
                    text_style: <REGULAR_FONT>{font_size: 12}
                    color: #000
                }
            }
            url = <MolyTextInput> {
                empty_message: "http://example.com/api"
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct BattleSection {
    #[deref]
    deref: View,
}

impl Widget for BattleSection {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);

        let url_input = self.text_input(id!(url));
        let preferences = scope.preferences_mut();

        if let Event::Actions(actions) = event {
            if let Some(current) = url_input.changed(actions) {
                preferences.battle_url = current;
                preferences.save();
            }
        }

        // mimic reactive binding
        if preferences.battle_url != url_input.text() {
            url_input.set_text(&preferences.battle_url);
            let end = preferences.battle_url.len();
            url_input.set_cursor(end, end);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}
