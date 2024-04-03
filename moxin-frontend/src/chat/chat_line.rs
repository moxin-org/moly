use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;

    ICON_COPY = dep("crate://self/resources/icons/copy.svg")
    ICON_EDIT = dep("crate://self/resources/icons/edit.svg")
    ICON_DELETE = dep("crate://self/resources/icons/delete.svg")

    ChatLine = {{ChatLine}} {
        margin: {top: 14, bottom: 3},
        width: Fill,
        height: Fit,

        cursor: Arrow,

        avatar_section = <View> {
            width: Fit,
            height: Fit,
            margin: {left: 20, right: 20},
        }

        main_section = <View> {
            width: Fill,
            height: Fit,

            flow: Down,
            spacing: 8,

            body_section = <View> {}

            actions_section = <View> {
                width: Fill,
                height: 14,
                actions = <View> {
                    width: Fill,
                    height: Fit,
                    visible: false,

                    <Icon> {
                        draw_icon: {
                            svg_file: (ICON_EDIT),
                            fn get_color(self) -> vec4 {
                                return #BDBDBD;
                            }
                        }
                        icon_walk: {width: 14, height: 14}
                    }
                    <Icon> {
                        draw_icon: {
                            svg_file: (ICON_COPY),
                            fn get_color(self) -> vec4 {
                                return #BDBDBD;
                            }
                        }
                        icon_walk: {width: 14, height: 14}
                    }
                    <Icon> {
                        draw_icon: {
                            svg_file: (ICON_DELETE),
                            fn get_color(self) -> vec4 {
                                return #BDBDBD;
                            }
                        }
                        icon_walk: {width: 14, height: 14}
                    }
                }
            }
        }

    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatLine {
    #[deref]
    view: View,
}

impl Widget for ChatLine {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let actions = cx.capture_actions(|cx| self.view.handle_event(cx, event, scope));
        if let Some(action) = actions.find_widget_action(self.view.widget_uid()) {
            if let ViewAction::FingerHoverIn(_) = action.cast() {
                self.view(id!(actions_section.actions))
                    .set_visible_and_redraw(cx, true);
            }
            if let ViewAction::FingerHoverOut(_) = action.cast() {
                self.view(id!(actions_section.actions))
                    .set_visible_and_redraw(cx, false);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
