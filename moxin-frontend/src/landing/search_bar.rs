use makepad_widgets::*;
use crate::data::store::{StoreAction, SortCriteria};
use crate::landing::sorting::SortingWidgetExt;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import makepad_draw::shader::std::*;

    import crate::landing::sorting::Sorting;

    ICON_SEARCH = dep("crate://self/resources/icons/search.svg")

    SearchBar = {{SearchBar}} {
        width: Fill,
        height: 200,

        flow: Down,
        spacing: 30,
        align: {x: 0.5, y: 0.5},

        show_bg: true,

        draw_bg: {
            color: #cccccc33,
            instance color2: #AF56DA55,
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

        input_container = <RoundedView> {
            width: Fit,
            height: Fit,

            show_bg: true,
            draw_bg: {
                color: #fff
            }

            padding: {top: 3, bottom: 3, left: 20, right: 20}

            spacing: 4,
            align: {x: 0.0, y: 0.5},

            draw_bg: {
                radius: 10.0,
                border_color: #D0D5DD,
                border_width: 1.0,
            }

            <Icon> {
                draw_icon: {
                    svg_file: (ICON_SEARCH),
                    fn get_color(self) -> vec4 {
                        return #666;
                    }
                }
                icon_walk: {width: 24, height: 24}
            }

            input = <TextInput> {
                width: 600,
                height: Fit,

                empty_message: "Search Model by Keyword"
                draw_bg: {
                    color: #fff
                }
                draw_text: {
                    text_style:<REGULAR_FONT>{font_size: 10},
                    fn get_color(self) -> vec4 {
                        return #555
                    }
                }

                // TODO find a way to override colors
                draw_cursor: {
                    instance focus: 0.0
                    uniform border_radius: 0.5
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.box(
                            0.,
                            0.,
                            self.rect_size.x,
                            self.rect_size.y,
                            self.border_radius
                        )
                        sdf.fill(mix(#fff, #bbb, self.focus));
                        return sdf.result
                    }
                }

                // TODO find a way to override colors
                draw_select: {
                    instance hover: 0.0
                    instance focus: 0.0
                    uniform border_radius: 2.0
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.box(
                            0.,
                            0.,
                            self.rect_size.x,
                            self.rect_size.y,
                            self.border_radius
                        )
                        sdf.fill(mix(#eee, #ddd, self.focus)); // Pad color
                        return sdf.result
                    }
                }
            }
        }

        search_sorting = <View> {
            visible: false,
            width: Fit,
            height: Fit,
            <Sorting> {}
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct SearchBar {
    #[deref]
    view: View
}

impl Widget for SearchBar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}


impl WidgetMatchEvent for SearchBar {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        if let Some(keywords) = self.text_input(id!(input)).changed(actions) {
            if keywords.len() > 3 {
                let widget_uid = self.widget_uid();
                cx.widget_action(
                    widget_uid,
                    &scope.path,
                    StoreAction::Search(keywords.to_string()),
                );
            } else {
                let widget_uid = self.widget_uid();
                cx.widget_action(
                    widget_uid,
                    &scope.path,
                    StoreAction::ResetSearch,
                );
            }
        }
    }
}

impl SearchBarRef {
    pub fn collapse(&self, cx: &mut Cx, selected_sort: SortCriteria) {
        let Some(mut inner) = self.borrow_mut() else { return };
        inner.apply_over(cx, live!{
            flow: Right,
            title = { visible: false }
            height: 100,
            align: {x: 0.0, y: 0.5},
            padding: {left: 20},
            spacing: 80,
            search_sorting = { visible: true }
        });

        inner.sorting(id!(search_sorting)).set_selected_item(selected_sort);
    }

    pub fn expand(&self, cx: &mut Cx) {
        let Some(mut inner) = self.borrow_mut() else { return };
        inner.apply_over(cx, live!{
            flow: Down,
            title = { visible: true }
            height: 200,
            align: {x: 0.5, y: 0.5},
            padding: {left: 0},
            spacing: 50,
            search_sorting = { visible: false }
        })
    }
}