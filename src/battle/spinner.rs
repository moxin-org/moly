use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;

    Spinner = <View> {
        height: Fit,
        width: Fit,
        <Label> {
            draw_text: {
                text_style: {font_size: 14},
                color: #000
            }
            text: "Spinner at home"
        }
    }
}
