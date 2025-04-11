use crate::data::search::SearchAction;
use crate::data::store::{Store, StoreAction};
use crate::landing::model_list::ModelListAction;
use crate::landing::search_bar::SearchBarWidgetExt;
use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::landing::search_bar::SearchBar;
    use crate::landing::model_list::ModelList;
    use crate::landing::downloads::Downloads;

    pub LandingScreen = {{LandingScreen}} {
        width: Fill,
        height: Fill,
        flow: Down,

        search_bar = <SearchBar> {}

        models = <View> {
            width: Fill,
            height: Fill,
            flow: Down,
            spacing: 30,
            padding: {left: 30, right: 30}

            heading_with_filters = <View> {
                width: Fit,
                height: 50,
                padding: {top: 30},

                align: {x: 0.5, y: 0.5},

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

            <ModelList> {}
        }
        downloads = <Downloads> {}
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum SearchBarState {
    #[default]
    ExpandedWithoutFilters,
    ExpandedWithFilters,
    CollapsedWithoutFilters,
    CollapsedWithFilters,
}

#[derive(Live, LiveHook, Widget)]
pub struct LandingScreen {
    #[deref]
    view: View,

    #[rust]
    search_bar_state: SearchBarState,
}

impl Widget for LandingScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let search = &scope.data.get::<Store>().unwrap().search;
        if search.is_pending() || search.was_error() {
            self.view(id!(heading_with_filters)).set_visible(cx, false);
        } else if let Some(keyword) = search.keyword.clone() {
            self.view(id!(heading_with_filters)).set_visible(cx, true);

            let models = &search.models;
            let models_count = models.len();
            self.label(id!(heading_with_filters.results))
                .set_text(cx, &format!("{} Results", models_count));
            self.label(id!(heading_with_filters.keyword))
                .set_text(cx, &format!(" for \"{}\"", keyword));
        } else {
            self.view(id!(heading_with_filters)).set_visible(cx, false);
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for LandingScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        for action in actions.iter() {
            if let Some(_) = action.downcast_ref::<SearchAction>() {
                self.redraw(cx);
            }

            match action.cast() {
                ModelListAction::ScrolledAtTop => {
                    if self.search_bar_state == SearchBarState::CollapsedWithoutFilters {
                        self.search_bar_state = SearchBarState::ExpandedWithoutFilters;
                        self.search_bar(id!(search_bar)).expand(cx);
                        self.redraw(cx);
                    }
                }
                ModelListAction::ScrolledNotAtTop => {
                    let collapse: bool;
                    match self.search_bar_state {
                        SearchBarState::ExpandedWithoutFilters => {
                            self.search_bar_state = SearchBarState::CollapsedWithoutFilters;
                            collapse = true;
                        }
                        SearchBarState::ExpandedWithFilters => {
                            self.search_bar_state = SearchBarState::CollapsedWithFilters;
                            collapse = true;
                        }
                        _ => {
                            collapse = false;
                        }
                    }

                    if collapse {
                        let search = &scope.data.get::<Store>().unwrap().search;
                        self.search_bar(id!(search_bar))
                            .collapse(cx, search.sorted_by);
                        self.redraw(cx);
                    }
                }
                _ => {}
            }

            match action.cast() {
                StoreAction::Search(_keywords) => match self.search_bar_state {
                    SearchBarState::CollapsedWithoutFilters => {
                        self.search_bar_state = SearchBarState::CollapsedWithFilters;
                    }
                    SearchBarState::ExpandedWithoutFilters => {
                        self.search_bar_state = SearchBarState::ExpandedWithFilters;
                    }
                    _ => {}
                },
                StoreAction::ResetSearch => match self.search_bar_state {
                    SearchBarState::ExpandedWithFilters | SearchBarState::CollapsedWithFilters => {
                        self.search_bar_state = SearchBarState::ExpandedWithoutFilters;
                        self.search_bar(id!(search_bar)).expand(cx);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
