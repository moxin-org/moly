use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;

    ModelCard = <View> {
        <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 14},
                color: #f00
            }
            text: "Model"
        }
    }
}