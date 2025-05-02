use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    use makepad_code_editor::code_view::CodeView;

    MESSAGE_FONT_SIZE = 10.5;
    MESSAGE_TEXT_COLOR = #x0

    pub MessageMarkdown = <CustomMarkdown> {
        padding: 0,
        font_color: #000,
        width: Fill, height: Fit,

        align: { y: 0.5 }
        font_size: (MESSAGE_FONT_SIZE),
        font_color: (MESSAGE_TEXT_COLOR),
        draw_normal:      { color: (MESSAGE_TEXT_COLOR), }
        draw_italic:      { color: (MESSAGE_TEXT_COLOR), }
        draw_bold:        { color: (MESSAGE_TEXT_COLOR), }
        draw_bold_italic: { color: (MESSAGE_TEXT_COLOR), }
        draw_fixed:       { color: (MESSAGE_TEXT_COLOR), }
        draw_block: {
            line_color: (MESSAGE_TEXT_COLOR)
            sep_color: (MESSAGE_TEXT_COLOR)
            code_color: (#EDEDED)
            quote_bg_color: (#EDEDED)
            quote_fg_color: (MESSAGE_TEXT_COLOR)
        }
        list_item_layout: { padding: {left: 5.0, top: 1.0, bottom: 1.0}, }
        list_item_walk: { margin: { left: 0, right: 0, top: 2, bottom: 4 } }
        code_layout: { padding: {top: 10.0, bottom: 0.0, left: 10.0, right: 10.0} }
        code_walk: { margin: { top: 0, bottom: 10, left: 0, right: 0 } }
        quote_layout: { spacing: 10, padding: {top: 0.0, bottom: 0.0}, }
        quote_walk: { margin: { top: 5, bottom: 5 } }
        inline_code_padding: {top: 3, bottom: 3, left: 4, right: 4 }
        inline_code_margin: { left: 3, right: 3, bottom: 3, top: 2 }
    }
}
