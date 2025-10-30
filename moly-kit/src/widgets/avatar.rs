//! The avatar of a bot in a chat message.

use crate::protocol::*;
use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use link::moly_kit_theme::*;
    use link::shaders::*;

    pub Avatar = {{Avatar}} <View> {
        height: Fit,
        width: Fit,
        grapheme = <RoundedView> {
            visible: false,
            width: 24,
            height: 24,

            show_bg: true,
            draw_bg: {
                color: #37567d,
                border_radius: 3,
            }

            align: {x: 0.5, y: 0.5},

            label = <Label> {
                width: Fit,
                height: Fit,
                draw_text:{
                    text_style: <THEME_FONT_BOLD>{font_size: 8.5},
                    color: #fff,
                }
                text: "P"
            }
        }

        dependency = <RoundedView> {
            width: 28, height: 28
            visible: false

            show_bg: true
            draw_bg: {
                border_radius: 2
            }

            image = <Image> {
                width: 28, height: 28
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
                    self.view(ids!(grapheme)).set_visible(cx, true);
                    self.view(ids!(dependency)).set_visible(cx, false);
                    self.label(ids!(label)).set_text(cx, &grapheme);
                }
                Picture::Dependency(dependency) => {
                    self.view(ids!(dependency)).set_visible(cx, true);
                    self.view(ids!(grapheme)).set_visible(cx, false);
                    let _ = self
                        .image(ids!(image))
                        .load_image_dep_by_path(cx, dependency.as_str());
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
