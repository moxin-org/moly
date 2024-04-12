use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;

    Header = <View> {
        width: Fill,
        height: Fit,
        spacing: 15,

        <Label> {
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 11},
                color: #000
            }
            text: "Model Downloads"
        }

        <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 11},
                color: #099250
            }
            text: "1 downloading"
        }

        <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 11},
                color: #667085
            }
            text: "1 paused"
        }

        <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 11},
                color: #667085
            }
            text: "5 completed"
        }
    }

    Content = <View> {
        width: Fill,
        height: 350,

        flow: Down,

        <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 11},
                color: #667085
            }
            text: "Downloading"
        }

        <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 11},
                color: #667085
            }
            text: "Paused"
        }

        <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 11},
                color: #667085
            }
            text: "Completed"
        }
    }

    Downloads = {{Downloads}} {
        width: Fill,
        height: Fit,
        flow: Down,

        show_bg: true,
        draw_bg: {
            color: #FCFCFD,
        }

        // TODO there is a better way to have only top-border?
        <Line> { draw_bg: { color: #EAECF0 }}
        <Header> {
            padding: {top: 12.0, bottom: 12.0, left: 43.0},
        }
        content = <Content> {
            height: 0,
            padding: {top: 12.0, bottom: 12.0, left: 43.0},
        }

        animator: {
            content = {
                default: collapse,
                expand = {
                    redraw: true,
                    from: {all: Forward {duration: 0.3}}
                    ease: ExpDecay {d1: 0.80, d2: 0.97}
                    apply: {content = { height: 350.0 }}
                }
                collapse = {
                    redraw: true,
                    from: {all: Forward {duration: 0.3}}
                    ease: ExpDecay {d1: 0.80, d2: 0.97}
                    apply: {content = { height: 0.0 }}
                }
            }
        }

    }
}

#[derive(Live, LiveHook, Widget)]
pub struct Downloads {
    #[deref]
    view: View,

    #[animator]
    animator: Animator,
}

impl Widget for Downloads {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        //self.widget_match_event(cx, event, scope);

        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }

        match event.hits(cx, self.view.area()) {
            Hit::FingerDown(_) => {
                self.animator_play(cx, id!(content.expand));
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl DownloadsRef {
    pub fn collapse(&mut self, cx: &mut Cx) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.animator_play(cx, id!(content.collapse));
    }
}
