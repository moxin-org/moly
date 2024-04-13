use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;

    import crate::landing::download_item::DownloadItem;

    Header = <View> {
        width: Fill,
        height: Fit,
        spacing: 25,

        <Label> {
            margin: {right: 20.0},
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 9},
                color: #000
            }
            text: "Model Downloads"
        }

        <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 9},
                color: #099250
            }
            text: "1 downloading"
        }

        <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 9},
                color: #667085
            }
            text: "1 paused"
        }

        <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 9},
                color: #667085
            }
            text: "5 completed"
        }
    }

    Content = <View> {
        width: Fill,
        height: 350,

        list = <PortalList> {
            width: Fill,
            height: Fill,

            DownloadItem = <DownloadItem> {}
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
            padding: {top: 20.0, bottom: 20.0, left: 43.0},
        }
        content = <Content> {
            height: 0,
            padding: {top: 12.0, bottom: 12.0, left: 43.0, right: 43.0},
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

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let downloads_count = 4;

        while let Some(view_item) = self.view.draw_walk(cx, &mut Scope::empty(), walk).step() {
            if let Some(mut list) = view_item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, downloads_count);
                while let Some(item_id) = list.next_visible_item(cx) {
                    let item = list.item(cx, item_id, live_id!(DownloadItem)).unwrap();

                    if item_id < downloads_count {
                        // item.draw_all(cx, &mut Scope::with_data(&mut model_data.clone()));
                        item.draw_all(cx, &mut Scope::with_data(&mut Scope::empty()));
                    }
                }
            }
        }

        DrawStep::done()
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
