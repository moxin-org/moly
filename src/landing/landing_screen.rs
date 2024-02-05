use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::landing::model_list::ModelList;

    LandingScreen = <View> {
        width: Fill,
        height: Fill,

        flow: Down,
        margin: 50,
        spacing: 30,

        <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 20},
                color: #f00
            }
            text: "LLM Studio"
        }

        <ModelList> {}
    }
}