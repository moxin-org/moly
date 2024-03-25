use makepad_widgets::*;
use moxin_protocol::data::Model;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import makepad_draw::shader::std::*;

    ModelAttributeTag = <RoundedView> {
        width: Fit,
        height: Fit,
        padding: {top: 6, bottom: 6, left: 10, right: 10}

        spacing: 5,
        draw_bg: {
            radius: 2.0,
        }

        caption = <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 9},
                color: #fff
            }
        }
    }

    ModelInfo = <View> {
        width: Fill,
        height: Fit,
        padding: 14,
        spacing: 5,
        align: {x: 0.0, y: 0.5},

        cursor: Hand,

        show_bg: true,
        draw_bg: {
            instance hover: 0.0,
            color: #fff,
            instance color_hover: #F9FAFB,

            fn pixel(self) -> vec4 {
                return mix(self.color, self.color_hover, self.hover);
            }
        }

        tag1 = <ModelAttributeTag> {
            caption = {
                text: "StableLM"
            }
            draw_bg: {
                color: #A44EBB,
            }
        }

        tag2 = <ModelAttributeTag> {
            caption = {
                text: "3B"
            }
            draw_bg: {
                color: #44899A,
            }
        }

        tag3 = <ModelAttributeTag> {
            caption = {
                text: "1.62 GB",
                draw_text:{
                    color: #000
                }
            }
            draw_bg: {
                color: #fff,
                border_color: #B4B4B4,
                border_width: 1.0,
            }
        }

        label = <Label> {
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 11},
                color: #000
            }
            text: "Model"
        }
    }

    ModelSelectorList = {{ModelSelectorList}} {
        flow: Down,
        template: <ModelInfo> {
            animator: {
                hover = {
                    default: off
                    off = {
                        from: {all: Forward {duration: 0.2}}
                        apply: {
                            draw_bg: {hover: 0.0}
                        }
                    }
    
                    on = {
                        from: {all: Snap}
                        apply: {
                            draw_bg: {hover: 1.0}
                        },
                    }
                }
            }
        }
    }

    ModelSelector = {{ModelSelector}} {
        width: Fill,
        height: Fit,

        flow: Down,

        button = <RoundedView> {
            width: Fill,
            height: 54,

            align: {x: 0.5, y: 0.5},

            draw_bg: {
                instance radius: 3.0,
                color: #F9FAFB,
                border_color: #DFDFDF,
                border_width: 1.0,
            }

            cursor: Hand,

            choose = <View> {
                width: Fit,
                height: Fit,

                <Label> {
                    draw_text:{
                        text_style: <BOLD_FONT>{font_size: 11},
                        color: #000
                    }
                    text: "Choose a model"
                }
            }
            selected = <ModelInfo> {
                width: Fit,
                height: Fit,
                visible: false
            }
        }

        options = <RoundedView> {
            width: Fill,
            height: Fit,
            visible: false

            margin: { top: 5 },
            padding: 5,

            draw_bg: {
                instance radius: 3.0,
                color: #fff,
                border_color: #B6B6B6,
                border_width: 1.0,
            }

            <ModelSelectorList> {
                width: Fill,
                height: Fit,
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelSelector {
    #[deref]
    view: View,

    #[rust]
    open: bool
}

impl Widget for ModelSelector {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}


impl WidgetMatchEvent for ModelSelector {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        if let Some(fe) = self.view(id!(button)).finger_up(&actions) {
            if fe.was_tap() {
                self.open = !self.open;
                self.view(id!(options)).apply_over(cx, live!{
                    visible: (self.open)
                });
                self.redraw(cx);
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelSelectorList {
    #[redraw] #[rust]
    area: Area,

    #[walk]
    walk: Walk,

    #[layout]
    layout: Layout,

    #[live]
    template: Option<LivePtr>,

    #[live(true)]
    visible: bool,

    #[rust]
    items: ComponentMap<LiveId, WidgetRef>,
}

impl Widget for ModelSelectorList {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        for (id, item) in self.items.iter_mut() {
            let actions = cx.capture_actions(|cx| item.handle_event(cx, event, scope));
            if let Some(fd) = item.as_view().finger_down(&actions) {
                if fd.tap_count == 1 {
                    dbg!("tapped", id);
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        //let model = scope.data.get::<Model>();
        let items = [
            "stablelm-zephy-3b.Q5_K_M.gguf".to_string(),
            "Nexusraven-v2-13b.Q6_K.gguf".to_string(),
            "phi2.Q4_K_S.gguf".to_string(),
        ].to_vec();
        cx.begin_turtle(walk, self.layout);

        if self.visible {
            self.draw_items(cx, &items);
        }
        cx.end_turtle_with_area(&mut self.area);

        DrawStep::done()
    }
}

impl ModelSelectorList {
    fn draw_items(&mut self, cx: &mut Cx2d, items: &Vec<String>) {
        for i in 0..items.len() {
            let item_id = LiveId(i as u64).into();
            let item_widget = self.items.get_or_insert(cx, item_id, | cx | {
                WidgetRef::new_from_ptr(cx, self.template)
            });
            let caption = &items[i];
            dbg!(caption);
            item_widget.apply_over(cx, live!{
                label = { text: (caption) }
            });

            let _ = item_widget.draw_all(cx, &mut Scope::empty());
        }
    }
}
