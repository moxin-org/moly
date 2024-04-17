use makepad_widgets::*;

live_design! {
    const MODEL_LINK_FONT_COLOR = #x155EEF

    REGULAR_FONT = {
        font_size: (12),
        font: {path: dep("crate://self/resources/fonts/Inter-Regular.ttf")}
    }

    BOLD_FONT = {
        font_size: (12),
        font: {path: dep("crate://self/resources/fonts/Inter-Bold.ttf")}
    }
}
