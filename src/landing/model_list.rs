use makepad_widgets::*;
use crate::data::store::Store;

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
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let models = Store::new().models.clone();
        let models_count = models.len();

        while let Some(view_item) = self.view.draw_walk(cx, scope, walk).step(){
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