use makepad_widgets::*;
use crate::data::store::{Store, StoreAction, SortCriteria};
use crate::landing::search_bar::SearchBarWidgetExt;
use crate::landing::model_list::ModelListAction;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import makepad_draw::shader::std::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::VerticalFiller;
    import crate::landing::search_bar::SearchBar;
    import crate::landing::model_list::ModelList;
    import crate::landing::sorting::Sorting;

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

        sorting = <Sorting> {
            width: Fit,
            height: Fit,
            visible: false
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

            spacing: 30,

            search_bar = <SearchBar> {}
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

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum SearchBarState {
    #[default] ExpandedWithoutFilters,
    ExpandedWithFilters,
    CollapsedWithoutFilters,
    CollapsedWithFilters
}

#[derive(Live, LiveHook, Widget)]
pub struct LandingScreen {
    #[deref]
    view: View,

    #[rust]
    search_bar_state: SearchBarState
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

            if self.search_bar_state == SearchBarState::ExpandedWithoutFilters {
                self.search_bar_state = SearchBarState::ExpandedWithFilters;
                self.view(id!(sorting)).set_visible(true);
            }
            if self.search_bar_state == SearchBarState::CollapsedWithoutFilters {
                self.search_bar_state = SearchBarState::CollapsedWithFilters;
            }

            let models = &store.models;
            let models_count = models.len();
            self.label(id!(heading_with_filters.results)).set_text(&format!("{} Results", models_count));
            self.label(id!(heading_with_filters.keyword)).set_text(&format!(" for \"{}\"", keyword));
        } else {
            self.view(id!(heading_with_filters)).set_visible(false);
            self.view(id!(heading_no_filters)).set_visible(true);

            // Keyword was removed from the search input
            if self.search_bar_state == SearchBarState::CollapsedWithFilters ||
                self.search_bar_state == SearchBarState::ExpandedWithFilters {
                    self.search_bar_state = SearchBarState::ExpandedWithoutFilters;
                    self.search_bar(id!(search_bar)).expand(cx);
                    self.view(id!(sorting)).set_visible(false);
            }
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

        let store = scope.data.get::<Store>();
        for action in actions.iter() {
            match action.as_widget_action().cast() {
                ModelListAction::ScrolledAtTop => {
                    if self.search_bar_state == SearchBarState::CollapsedWithoutFilters {
                        self.search_bar_state = SearchBarState::ExpandedWithoutFilters;
                        self.search_bar(id!(search_bar)).expand(cx);
                        self.view(id!(sorting)).set_visible(false);
                        self.redraw(cx);
                    }
                }
                ModelListAction::ScrolledNotAtTop => {
                    if self.search_bar_state == SearchBarState::ExpandedWithoutFilters {
                        self.search_bar_state = SearchBarState::CollapsedWithoutFilters;
                    } else if self.search_bar_state == SearchBarState::ExpandedWithFilters {
                        self.search_bar_state = SearchBarState::CollapsedWithFilters;
                    }
                    self.search_bar(id!(search_bar)).collapse(cx);
                    self.view(id!(sorting)).set_visible(false);
                    self.redraw(cx);
                }
                _ => {}
            }
        }
    }
}