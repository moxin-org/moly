use makepad_widgets::*;
use moxin_protocol::data::FileID;

use super::modal::ModalAction;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::resource_imports::*;
    import crate::landing::shared::*;
    import makepad_draw::shader::std::*;

    SUCCESS_ICON = dep("crate://self/resources/images/success_icon.png")
    ERROR_ICON = dep("crate://self/resources/images/error_icon.png")

    DownloadNotificationPopup = {{DownloadNotificationPopup}} {
        width: Fit
        height: Fit

        <RoundedView> {
            width: 350
            height: Fit
            margin: {top: 20, right: 20}
            padding: {top: 20, right: 20 bottom: 20 left: 20}
            spacing: 15

            show_bg: true
            draw_bg: {
                color: #fff
                instance border_radius: 4.0
                fn pixel(self) -> vec4 {
                    let border_color = #d4;
                    let border_width = 1;
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    let body = #fff

                    sdf.box(
                        1.,
                        1.,
                        self.rect_size.x - 2.0,
                        self.rect_size.y - 2.0,
                        self.border_radius
                    )
                    sdf.fill_keep(body)

                    sdf.stroke(
                        border_color,
                        border_width
                    )
                    return sdf.result
                }
            }

            success_icon = <Image> {
                margin: {top: -10, left: -10}
                source: (SUCCESS_ICON),
                width: 35,
                height: 35,
            }

            <View> {
                width: Fill,
                height: Fit,
                flow: Down,
                spacing: 10,

                <View> {
                    flow: Right
                    width: Fill,
                    height: Fit,

                    title = <Label> {
                        draw_text:{
                            text_style: <BOLD_FONT>{font_size: 9},
                            word: Wrap,
                            color: #000
                        }
                        text: "Model Downloaded Successfully"
                    }

                    filler_x = <View> {width: Fill, height: Fit}

                    close_button = <RoundedView> {
                        width: Fit,
                        height: Fit,
                        align: {x: 0.5, y: 0.5}
                        cursor: Hand

                        button_icon = <Icon> {
                            draw_icon: {
                                svg_file: (ICON_CLOSE),
                                fn get_color(self) -> vec4 {
                                    return #000;
                                }
                            }
                            icon_walk: {width: 10, height: 10}
                        }
                    }
                }

                summary = <Label> {
                    width: Fill,
                    draw_text:{
                        text_style: <REGULAR_FONT>{font_size: 9},
                        word: Wrap,
                        color: #000
                    }
                    text: ""
                }
                view_in_my_models_button = <ModelLink> {
                    link = { text: "View in My Models" }
                }
            }

        }
    }

}

#[derive(Clone, DefaultNone, Eq, Hash, PartialEq, Debug)]
pub enum PopupAction {
    None,
    NavigateToMyModels,
}

#[derive(Live, LiveHook, Widget)]
pub struct DownloadNotificationPopup {
    #[deref]
    view: View,
    #[layout]
    layout: Layout,
    #[rust]
    file_id: FileID,
}

impl Widget for DownloadNotificationPopup {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let display_file_name = self.file_id.split('#').collect::<Vec<_>>()[1].to_string();

        self.label(id!(summary))
            .set_text(&(format!("{} successfuly downloaded.", &display_file_name)));

        let _ = self
            .view
            .draw_walk(cx, scope, walk.with_abs_pos(DVec2 { x: 0., y: 0. }));

        DrawStep::done()
    }
}

impl WidgetMatchEvent for DownloadNotificationPopup {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let widget_uid = self.widget_uid();

        let view_in_my_models_button = self.link_label(id!(view_in_my_models_button.link));
        if view_in_my_models_button.clicked(actions) {
            // TODO: Abstract the navigation actions on a single enum for the whole app.
            cx.widget_action(widget_uid, &scope.path, PopupAction::NavigateToMyModels);
            cx.widget_action(widget_uid, &scope.path, ModalAction::CloseModal);
        }

        if let Some(fe) = self.view(id!(close_button)).finger_up(actions) {
            if fe.was_tap() {
                cx.widget_action(widget_uid, &scope.path, ModalAction::CloseModal);
            }
        }
    }
}

impl DownloadNotificationPopup {
    pub fn set_file_id(&mut self, file_id: FileID) {
        self.file_id = file_id;
    }
}

impl DownloadNotificationPopupRef {
    pub fn set_file_id(&mut self, file_id: FileID) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_file_id(file_id)
        }
    }
}
