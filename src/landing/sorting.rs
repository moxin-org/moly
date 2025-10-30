use crate::data::{search::SortCriteria, store::StoreAction};
use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;

    ModelsDropDown = <DropDown> {
        width: Fit
        height: Fit
        padding: {top: 20.0, right: 10.0, bottom: 20.0, left: 16.0}

        popup_menu_position: BelowInput

        draw_text: {
            text_style: <BOLD_FONT> { font_size: 9 },
            fn get_color(self) -> vec4 {
                return mix(
                    #000,
                    #000,
                    self.focus
                )
            }
        }

        popup_menu: {
            width: 220,

            draw_bg: {
                color: #fff,
                border_size: 1.5,
                //border_color_1: #EAECF0,
                border_radius: 4.0
            }

            menu_item: {
                width: Fill,
                height: Fit

                padding: {left: 20, top: 15, bottom: 15, right: 20}

                draw_bg: {
                    color: #fff,
                    color_active: #eee9,

                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);

                        sdf.clear(mix(
                            self.color,
                            self.color_active,
                            self.hover
                        ))

                        let sz = 3.;
                        let dx = 1.6;
                        let c = vec2(0.9 * self.rect_size.x, 0.5 * self.rect_size.y);
                        sdf.move_to(c.x - sz + dx * 0.5, c.y - sz + dx);
                        sdf.line_to(c.x, c.y + sz);
                        sdf.line_to(c.x + sz * 2.0, c.y - sz * 2.0);
                        sdf.stroke(mix(#0000, #0, self.active), 1.0);

                        return sdf.result;
                    }
                }

                draw_text: {
                    text_style: <BOLD_FONT> { font_size: 9 }
                    instance active: 0.0
                    instance hover: 0.0
                    fn get_color(self) -> vec4 {
                        return #000;
                    }
                }
            }
        }

        draw_bg: {
            instance open: 0.0

            fn get_bg(self, inout sdf: Sdf2d) {
                sdf.box(
                    2,
                    2,
                    self.rect_size.x - 4,
                    self.rect_size.y - 4,
                    4.0
                )
                sdf.stroke_keep(#EAECF0, 2.);
                sdf.fill(#fff);
            }

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                self.get_bg(sdf);

                let c = vec2(self.rect_size.x - 20.0, self.rect_size.y * 0.5)
                let sz = 2.5;

                if self.open < 0.5 {
                    sdf.move_to(c.x - sz * 2.0, c.y - sz);
                    sdf.line_to(c.x, c.y + sz);
                    sdf.line_to(c.x + sz * 2.0, c.y - sz);
                }
                else {
                    sdf.move_to(c.x - sz * 2.0, c.y + sz);
                    sdf.line_to(c.x, c.y - sz);
                    sdf.line_to(c.x + sz * 2.0, c.y + sz);
                }
                sdf.stroke(#666, 1.0);

                return sdf.result
            }
        }
    }

    pub Sorting = {{Sorting}} {
        width: Fit,
        height: Fit,
        align: {x: 0.5, y: 0.5},

        <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 9},
                color: #667085
            }
            text: "SORT BY"
        }

        options = <ModelsDropDown> {
            width: 220,
            height: Fit,

            margin: { left: 20, right: 40 }

            labels: ["Most Downloads", "Least Downloads", "Most Likes", "Least Likes"]
            values: [MostDownloads, LeastDownloads, MostLikes, LeastLikes]
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct Sorting {
    #[deref]
    view: View,
}

impl Widget for Sorting {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for Sorting {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        if let Some(item_selected) = self.drop_down(ids!(options)).selected(&actions) {
            // TODO Check if we can use liveids instead of item index
            let criteria = match item_selected {
                0 => SortCriteria::MostDownloads,
                1 => SortCriteria::LeastDownloads,
                2 => SortCriteria::MostLikes,
                3 => SortCriteria::LeastLikes,
                4_usize.. => panic!(),
            };

            cx.action(StoreAction::Sort(criteria));
        }
    }
}

impl SortingRef {
    pub fn _set_visible(&self, cx: &mut Cx, visible: bool) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.apply_over(
            cx,
            live! {
                visible: (visible)
            },
        );
    }

    pub fn set_selected_item(&self, cx: &mut Cx, criteria: SortCriteria) {
        let Some(inner) = self.borrow_mut() else {
            return;
        };
        let criteria_id = match criteria {
            SortCriteria::MostDownloads => 0,
            SortCriteria::LeastDownloads => 1,
            SortCriteria::MostLikes => 2,
            SortCriteria::LeastLikes => 3,
        };
        inner
            .drop_down(ids!(options))
            .set_selected_item(cx, criteria_id);
    }
}
