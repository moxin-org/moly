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

    Downloads = {{Downloads}} {
        width: Fill,
        height: Fit,
        flow: Down,

        show_bg: true,
        draw_bg: {
            color: #FCFCFD,
        }

        <Line> {
            draw_bg: {
                color: #EAECF0
            }
        }
        <Header> {
            padding: {top: 12.0, bottom: 12.0, left: 43.0},
        }

    }
}

#[derive(Live, LiveHook, Widget)]
pub struct Downloads {
    #[deref]
    view: View,
}

impl Widget for Downloads {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        //self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
