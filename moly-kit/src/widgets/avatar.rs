//! The avatar of a bot in a chat message.

use crate::protocol::*;
use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    pub Avatar = {{Avatar}} <View> {
        height: Fit,
        width: Fit,
        grapheme = <RoundedView> {
            visible: false,
            width: 24,
            height: 24,

            show_bg: true,
            draw_bg: {
                color: #444D9A,
                radius: 6,
            }

            align: {x: 0.5, y: 0.5},

            label = <Label> {
                width: Fit,
                height: Fit,
                draw_text:{
                    // text_style: <BOLD_FONT>{font_size: 10},
                    color: #fff,
                }
                text: "P"
            }
        }
    }
}

#[derive(Live, Widget, LiveHook)]
pub struct Avatar {
    #[deref]
    deref: View,

    #[rust]
    pub avatar: Option<Picture>,
}

impl Widget for Avatar {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if let Some(avatar) = &self.avatar {
            match avatar {
                Picture::Grapheme(grapheme) => {
                    self.view(id!(grapheme)).set_visible(cx, true);
                    self.label(id!(label)).set_text(cx, &grapheme);
                }
                _ => unimplemented!(),
            }
        }

        self.deref.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope)
    }
}
