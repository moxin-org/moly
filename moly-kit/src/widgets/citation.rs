use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    pub Citation = {{Citation}} <RoundedView> {
        flow: Down,
        height: Fit,
        width: 200,
        padding: 6,
        spacing: 4,
        draw_bg: {
            color: #f0f0f0
            radius: 3
        }

        <View> {
            height: Fit,
            align: {y: 0.5},
            icon = <Image> {
                width: 16,
                height: 16,
                source: dep("crate://self/resources/link.png")
            }

            site = <Label> {
                text: "3.basecamp.com",
                draw_text: {
                    color: #555,
                }
            }
        }

        <View> {
            height: Fit,
            title = <Label> {
                text: "[MolyKit] Complete URL preview feature",
                draw_text: {
                    color: #000,
                }
            }
        }
    }
}

#[derive(Live, Widget, LiveHook)]
pub struct Citation {
    #[deref]
    deref: View,
}

impl Widget for Citation {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope)
    }
}
