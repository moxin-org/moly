use makepad_widgets::*;
use crate::data::store::Store;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::landing::search_bar::SearchBar;
    import crate::landing::model_list::ModelList;

    LandingScreen = {{LandingScreen}} {
        width: Fill,
        height: Fill,
        flow: Overlay,

        <View> {
            width: Fill,
            height: Fill,
            flow: Down,

            <SearchBar> {}

            <View> {
                width: Fill,
                height: Fill,
                flow: Down,
                margin: 50,
                spacing: 30,

                heading_no_filters = <View> {
                    width: Fit,
                    height: Fit,
                    <Label> {
                        draw_text:{
                            text_style: <REGULAR_FONT>{font_size: 20},
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
                            text_style: <BOLD_FONT>{font_size: 18},
                            color: #000
                        }
                        text: "12 Results"
                    }
                    keyword = <Label> {
                        draw_text:{
                            text_style: <REGULAR_FONT>{font_size: 18},
                            color: #000
                        }
                        text: " for \"Open Hermes\""
                    }
                }

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
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get::<Store>();
        if let Some(keyword) = store.keyword.clone() {
            self.view(id!(heading_with_filters)).set_visible(true);
            self.view(id!(heading_no_filters)).set_visible(false);

            let models = &store.models;
            let models_count = models.len();

            self.label(id!(heading_with_filters.results)).set_text(&format!("{} Results", models_count));
            self.label(id!(heading_with_filters.keyword)).set_text(&format!(" for \"{}\"", keyword));
        } else {
            self.view(id!(heading_with_filters)).set_visible(false);
            self.view(id!(heading_no_filters)).set_visible(true);
        }

        self.view.draw_walk(cx, scope, walk)
    }
}