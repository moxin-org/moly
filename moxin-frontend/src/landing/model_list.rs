use makepad_widgets::*;
use crate::data::store::{Store, StoreAction};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::landing::model_card::ModelCard;

    ModelList = {{ModelList}} {
        width: Fill,
        height: Fill,

        list = <PortalList> {
            width: Fill,
            height: Fill,

            // We need this setting because we will have modal dialogs that should
            // "capture" the events, so we don't want to handle them here.
            capture_overload: false,

            Model = <ModelCard> {
                margin: {bottom: 30},
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelList {
    #[deref]
    view: View
}

impl Widget for ModelList {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get::<Store>().unwrap();
        let models = &store.models;
        let models_count = models.len();

        while let Some(view_item) = self.view.draw_walk(cx, &mut Scope::empty(), walk).step(){
            if let Some(mut list) = view_item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, models_count);
                while let Some(item_id) = list.next_visible_item(cx) {
                    let item = list.item(cx, item_id, live_id!(Model)).unwrap();

                    if item_id < models_count {
                        let model_data = &models[item_id];
                        item.draw_all(cx, &mut Scope::with_data(&mut model_data.clone()));
                    }
                }
            }
        }

        DrawStep::done()
    }
}

#[derive(Clone, DefaultNone, Debug)]
pub enum ModelListAction {
    None,
    ScrolledAtTop,
    ScrolledNotAtTop,
}

const SCROLLING_AT_TOP_THRESHOLD: f64 = -30.0;

impl WidgetMatchEvent for ModelList {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let portal_list_ref = self.portal_list(id!(list));
        for action in actions.iter() {
            match action.as_widget_action().cast() {
                StoreAction::Search(_keywords) => {
                    portal_list_ref.set_first_id_and_scroll(0, 0.0);
                }
                StoreAction::ResetSearch => {
                    portal_list_ref.set_first_id_and_scroll(0, 0.0);
                }
                _ => {}
            }
        }

        if portal_list_ref.scrolled(actions) {
            let widget_uid = self.widget_uid();
            if portal_list_ref.first_id() == 0 &&
                portal_list_ref.scroll_position() > SCROLLING_AT_TOP_THRESHOLD {
                    cx.widget_action(
                        widget_uid,
                        &scope.path,
                        ModelListAction::ScrolledAtTop,
                    );
            } else {
                cx.widget_action(
                    widget_uid,
                    &scope.path,
                    ModelListAction::ScrolledNotAtTop,
                );
            }
        }
    }
}