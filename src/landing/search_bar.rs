use crate::data::search::SortCriteria;
use crate::data::store::StoreAction;
use crate::landing::sorting::SortingWidgetExt;
use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::landing::sorting::Sorting;

    ICON_SEARCH = dep("crate://self/resources/icons/search.svg")
    ICON_CLOSE = dep("crate://self/resources/icons/close.svg")

    pub SearchBar = {{SearchBar}} {
        width: Fill,
        height: 200,

        flow: Down,
        spacing: 30,
        align: {x: 0.5, y: 0.5},

        show_bg: true,

        draw_bg: {
            color: (MAIN_BG_COLOR_DARK),
            instance color2: #a6bec6,
            fn get_color(self) -> vec4 {
                let coef = self.rect_size.y / self.rect_size.x;

                let distance_vec = self.pos - vec2(0.8, 1.1);
                let norm_distance = length(vec2(distance_vec.x, distance_vec.y * coef) * 2.2);

                if pow(norm_distance, 1.4) > 1.0 {
                    return self.color;
                } else {
                    return mix(self.color2, self.color, pow(norm_distance, 1.4));
                }
            }

            fn pixel(self) -> vec4 {
                return Pal::premul(self.get_color());
            }
        }

        title = <View> {
            width: Fit,
            height: Fit,
            <Label> {
                draw_text:{
                    text_style: <REGULAR_FONT>{font_size: 13},
                    color: #000
                }
                text: "Discover, download, and run local LLMs"
            }
        }

        input_container = <RoundedShadowView> {
            width: 800,
            height: Fit,

            show_bg: true,
            draw_bg: {
                color: (MAIN_BG_COLOR),
                border_radius: 8.5,
                uniform shadow_color: #0001
                shadow_radius: 8.0,
                shadow_offset: vec2(0.0,-2.0)
            }

            padding: {top: 3, bottom: 3, left: 20, right: 20}
            margin: {left: 30, right: 30}

            spacing: 4,
            align: {x: 0.0, y: 0.5},

            <Icon> {
                draw_icon: {
                    svg_file: (ICON_SEARCH),
                    fn get_color(self) -> vec4 {
                        return #666;
                    }
                }
                icon_walk: {width: 17, height: 17}
            }

            input = <MolyTextInput> {
                width: Fill,
                height: Fit,
                empty_text: "Search Model by Keyword",
                draw_bg: {
                    color: (MAIN_BG_COLOR),
                }
            }

            clear_text_button = <MolyButton> {
                visible: false,
                draw_icon: {
                    svg_file: (ICON_CLOSE),
                    fn get_color(self) -> vec4 {
                        return #8;
                    }
                }
                icon_walk: {width: 10, height: 10}
            }
        }

        search_sorting = <View> {
            visible: false,
            width: 300,
            height: Fit,
            margin: {left: 30, right: 30},
            <Sorting> {}
        }

        animator: {
            search_bar = {
                default: expanded,
                collapsed = {
                    redraw: true,
                    from: {all: Forward {duration: 0.3}}
                    ease: ExpDecay {d1: 0.80, d2: 0.97}
                    apply: { height: 100 }
                }
                expanded = {
                    redraw: true,
                    from: {all: Forward {duration: 0.3}}
                    ease: ExpDecay {d1: 0.80, d2: 0.97}
                    apply: { height: 200 }
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct SearchBar {
    #[deref]
    view: View,

    #[animator]
    animator: Animator,

    #[rust]
    collapsed: bool,

    #[rust]
    search_timer: Timer,

    #[live(0.3)]
    search_debounce_time: f64,
}

impl Widget for SearchBar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }

        if self.search_timer.is_event(event).is_some() {
            self.search_timer = Timer::default();

            let input = self.text_input(id!(input));
            let keywords = input.text();
            const MIN_SEARCH_LENGTH: usize = 2;

            if keywords.len() > MIN_SEARCH_LENGTH {
                cx.action(StoreAction::Search(keywords.to_string()));
            } else if keywords.len() == 0 {
                cx.action(StoreAction::ResetSearch);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for SearchBar {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let input = self.text_input(id!(input));
        let clear_text_button = self.button(id!(clear_text_button));

        if let Some(keywords) = input.returned(actions) {
            if keywords.len() > 0 {
                cx.action(StoreAction::Search(keywords.to_string()));
            } else {
                cx.action(StoreAction::ResetSearch);
            }
        }

        if let Some(text) = input.changed(actions) {
            clear_text_button.set_visible(cx, !text.is_empty());
            cx.stop_timer(self.search_timer);
            self.search_timer = cx.start_timeout(self.search_debounce_time);
        }

        if self.button(id!(clear_text_button)).clicked(actions) {
            input.set_text(cx, "".to_string());
            clear_text_button.set_visible(cx, false);
            input.set_key_focus(cx);

            cx.action(StoreAction::ResetSearch);
        }
    }
}

impl SearchBarRef {
    pub fn collapse(&self, cx: &mut Cx, selected_sort: SortCriteria) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        if inner.collapsed {
            return;
        }
        inner.collapsed = true;

        inner.apply_over(
            cx,
            live! {
                flow: Right,
                title = { visible: false }
                align: {x: 0.0, y: 0.5},
                padding: {left: 20},
                spacing: 80,
                input_container = { width: Fill }
                search_sorting = { visible: true }
            },
        );

        inner
            .sorting(id!(search_sorting))
            .set_selected_item(cx, selected_sort);
        inner.animator_play(cx, id!(search_bar.collapsed));
    }

    pub fn expand(&self, cx: &mut Cx) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        if !inner.collapsed {
            return;
        }
        inner.collapsed = false;

        inner.apply_over(
            cx,
            live! {
                flow: Down,
                title = { visible: true }
                align: {x: 0.5, y: 0.5},
                padding: {left: 0},
                spacing: 50,
                input_container = { width: 800 }
                search_sorting = { visible: false }
            },
        );

        inner.animator_play(cx, id!(search_bar.expanded));
    }
}
