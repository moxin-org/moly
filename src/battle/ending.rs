use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::battle::styles::*;

    Ending = <View> {
        flow: Down,
        align: {x: 0.5, y: 0.5},
        <Icon> {
            margin: {bottom: (LG_GAP)},
            draw_icon: {
                svg_file: dep("crate://self/resources/icons/my_models.svg"),
                fn get_color(self) -> vec4 {
                    return #0d0;
                }
            }
            icon_walk: {width: 250, height: 250}
        }
        <Label> {
            draw_text: {
                color: #000,
                text_style: <BOLD_FONT> { font_size: 14 }
            }
            text: "You're done! Thank you for participating."
        }
    }
}
