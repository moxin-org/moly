use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::landing::model_card::ModelCard;

    ModelList = <View> {
        width: Fill,
        height: Fill,

        flow: Down
        spacing: 30,

        <ModelCard> {}
        <ModelCard> {}
    }
}