use makepad_widgets::*;
use crate::landing::model_card::ModelCardAction;
use crate::landing::model_all_files::ModelAllFilesWidgetRefExt;
use crate::data::store::Store;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::landing::model_list::ModelList;
    import crate::landing::model_all_files::ModelAllFiles;

    LandingScreen = {{LandingScreen}} {
        width: Fill,
        height: Fill,
        flow: Overlay,

        main = <View> {
            width: Fill,
            height: Fill,
            flow: Down,
            margin: 50,
            spacing: 30,

            <Label> {
                draw_text:{
                    text_style: <REGULAR_FONT>{font_size: 20},
                    color: #000
                }
                text: "Moxin"
            }

            <ModelList> {}
        }

        all_files_panel = <SlidePanel> {
            width: 923,
            height: Fill,

            side: Right,
            closed: 1.0,

            animator: {
                closed = {
                    default: on
                }
            }
            
            all_files = <ModelAllFiles> {
                show_bg: true
                draw_bg: {
                    color: #F2F4F7
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct LandingScreen {
    #[deref]
    view: View,

    #[rust]
    all_files_panel_open: bool
}

impl Widget for LandingScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.all_files_panel_open {
            self.view(id!(all_files_panel)).handle_event(cx, event, scope);

            match event.hits(cx, self.view.area()) {
                Hit::FingerDown(fe) => {
                    let screen_rect = fe.rect.size.x;
                    if let Size::Fixed(panel_width) = self.view(id!(all_files_panel)).walk(cx).width {
                        if fe.abs.x < screen_rect - panel_width  {
                            self.slide_panel(id!(all_files_panel)).close(cx);
                            self.all_files_panel_open = false;
                        }
                    }
                },
                _ => ()
            }
        } else {
            self.view.handle_event(cx, event, scope);
            self.widget_match_event(cx, event, scope);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for LandingScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        for action in actions {
            if let ModelCardAction::ViewAllFiles(model_id) = action.as_widget_action().cast() {
                if let Some(model) = Store::new().models.iter().find(|m| m.name == model_id) {
                    self.view(id!(all_files_panel)).model_all_files(id!(all_files)).set_model(model.clone());
                    self.slide_panel(id!(all_files_panel)).open(cx);
                    self.all_files_panel_open = true;
                }
            };
        }
    }
}