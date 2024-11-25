use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;

    ModelFilesListLabel = <RoundedView> {
        width: Fit,
        height: Fit,
        padding: {top: 6, bottom: 6, left: 10, right: 10}

        draw_bg: {
            instance radius: 2.0,
            color: #E6F1EC,
        }

        label = <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 9},
                color: #1C1917
            }
        }
    }

    pub ModelFilesTags = {{ModelFilesTags}} {
        width: Fit,
        height: Fit,
        flow: Right,
        spacing: 5,

        template: <ModelFilesListLabel> {}
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelFilesTags {
    #[redraw]
    #[rust]
    area: Area,

    #[walk]
    walk: Walk,

    #[layout]
    layout: Layout,

    #[live]
    template: Option<LivePtr>,

    #[rust]
    items: ComponentMap<LiveId, WidgetRef>,
}

impl Widget for ModelFilesTags {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        for (_id, item) in self.items.iter_mut() {
            item.handle_event(cx, event, scope);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        cx.begin_turtle(walk, self.layout);
        for (_id, item) in self.items.iter_mut() {
            let _ = item.draw_walk(cx, scope, walk);
        }
        cx.end_turtle_with_area(&mut self.area);
        DrawStep::done()
    }
}

impl ModelFilesTagsRef {
    pub fn set_tags(&self, cx: &mut Cx, tags: &Vec<String>) {
        let Some(mut tags_widget) = self.borrow_mut() else {
            return;
        };
        tags_widget.items.clear();
        for (i, tag) in tags.iter().enumerate() {
            let item_id = LiveId(i as u64).into();
            let item_widget = WidgetRef::new_from_ptr(cx, tags_widget.template);
            item_widget.apply_over(cx, live! {label = { text: (tag) }});
            tags_widget.items.insert(item_id, item_widget);
        }
    }
}
