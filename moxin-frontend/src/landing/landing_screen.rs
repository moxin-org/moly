use makepad_widgets::*;
use crate::data::store::{Store, StoreAction, SortCriteria};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import makepad_draw::shader::std::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::VerticalFiller;
    import crate::landing::search_bar::SearchBar;
    import crate::landing::model_list::ModelList;

    ModelsDropDown = <DropDown> {
        width: Fit
        height: Fit
        padding: {top: 10.0, right: 20.0, bottom: 10.0, left: 10.0}

        popup_menu_position: BelowInput

        draw_text: {
            text_style: <BOLD_FONT> { font_size: 10 },
            fn get_color(self) -> vec4 {
                return mix(
                    mix(
                        #000,
                        #000,
                        self.focus
                    ),
                    #000,
                    self.pressed
                )
            }
        }

        popup_menu: {
            width: 220,

            draw_bg: {
                color: #fff,
                border_width: 1.5,
                border_color: #EAECF0,
                radius: 4.0
                blur: 0.0
            }

            menu_item: {
                width: Fill,
                height: Fit

                padding: {left: 20, top: 15, bottom: 15, right: 20}

                draw_bg: {
                    color: #fff,
                    color_selected: #eee9,

                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);

                        sdf.clear(mix(
                            self.color,
                            self.color_selected,
                            self.hover
                        ))

                        let sz = 4.;
                        let dx = 2.0;
                        let c = vec2(0.9 * self.rect_size.x, 0.5 * self.rect_size.y);
                        sdf.move_to(c.x - sz + dx * 0.5, c.y - sz + dx);
                        sdf.line_to(c.x, c.y + sz);
                        sdf.line_to(c.x + sz * 2.0, c.y - sz * 2.0);
                        sdf.stroke(mix(#0000, #0, self.selected), 1.5);

                        return sdf.result;
                    }
                }

                draw_name: {
                    text_style: <BOLD_FONT> { font_size: 10 }
                    instance selected: 0.0
                    instance hover: 0.0
                    fn get_color(self) -> vec4 {
                        return #000;
                    }
                }
            }
        }

        draw_bg: {
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
        }
    }

    Heading = <View> {
        width: Fill,
        height: Fit,
        spacing: 30,

        align: {x: 0.5, y: 0.5},

        heading_no_filters = <View> {
            width: Fit,
            height: Fit,
            <Label> {
                draw_text:{
                    text_style: <REGULAR_FONT>{font_size: 18},
                    color: #000
                }
                text: "Explore"
            }
        }

        heading_with_filters = <View> {
            width: Fit,
            height: Fit,
            results = <Label> {
                draw_text:{
                    text_style: <BOLD_FONT>{font_size: 16},
                    color: #000
                }
                text: "12 Results"
            }
            keyword = <Label> {
                draw_text:{
                    text_style: <REGULAR_FONT>{font_size: 16},
                    color: #000
                }
                text: " for \"Open Hermes\""
            }
        }

        <VerticalFiller> {}

        filters = <View> {
            width: Fit,
            height: Fit,
            align: {x: 0.5, y: 0.5},

            <Label> {
                draw_text:{
                    text_style: <REGULAR_FONT>{font_size: 12},
                    color: #667085
                }
                text: "SORT BY"
            }

            sorting = <ModelsDropDown> {
                width: 220,
                height: Fit,

                margin: { left: 20, right: 40 }

                labels: ["Most Downloads", "Least Downloads", "Most Likes", "Least Likes"]
                values: [MostDownloads, LeastDownloads, MostLikes, LeastLikes]
            }
        }
    }

    LandingScreen = {{LandingScreen}} {
        width: Fill,
        height: Fill,
        flow: Overlay,

        <View> {
            width: Fill,
            height: Fill,
            flow: Down,

            //margin: 50,
            spacing: 30,

            <SearchBar> {}
            <View> {
                width: Fill,
                height: Fill,
                flow: Down,
                spacing: 30,
                margin: { left: 50, right: 50 },

                <Heading> {}
                <ModelList> {}
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct LandingScreen {
    #[deref]
    view: View
}

impl Widget for LandingScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get::<Store>();
        if let Some(keyword) = store.keyword.clone() {
            self.view(id!(heading_with_filters)).set_visible(true);
            self.view(id!(heading_no_filters)).set_visible(false);
            self.view(id!(filters)).set_visible(true);

            let models = &store.models;
            let models_count = models.len();

            self.label(id!(heading_with_filters.results)).set_text(&format!("{} Results", models_count));
            self.label(id!(heading_with_filters.keyword)).set_text(&format!(" for \"{}\"", keyword));
        } else {
            self.view(id!(heading_with_filters)).set_visible(false);
            self.view(id!(heading_no_filters)).set_visible(true);
            self.view(id!(filters)).set_visible(false);
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for LandingScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        if let Some(item_selected) = self.drop_down(id!(sorting)).selected(&actions) {
            // TODO Check if we can use liveids instead of item index
            let criteria = match item_selected {
                0 => SortCriteria::MostDownloads,
                1 => SortCriteria::LeastDownloads,
                2 => SortCriteria::MostLikes,
                3 => SortCriteria::LeastLikes,
                4_usize.. => panic!()
            };

            let widget_uid = self.widget_uid();
            cx.widget_action(
                widget_uid,
                &scope.path,
                StoreAction::Sort(criteria),
            );
        }
    }
}