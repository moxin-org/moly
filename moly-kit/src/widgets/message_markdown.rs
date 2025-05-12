use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use link::moly_kit_theme::*;

    use makepad_code_editor::code_view::CodeView;

    MD_LINE_SPACING = 1.5
    MD_FONT_COLOR = #000

    pub MessageMarkdown = <Markdown> {
        padding: 0,
        margin: 0,
        paragraph_spacing: 16,
        heading_base_scale: 1.6

        font_color: #000,
        width: Fill, height: Fit,
        font_size: 11.0,
        code_block = <View> {
            width: Fill,
            height: Fit,
            flow: Down
            <RoundedView>{
                draw_bg: {
                    border_radius: 0.0
                    border_size: 1.2
                    border_color: #1d2330
                }
                width:Fill,
                height:Fit,
                align:{ x: 1.0 }
                
                copy_code_button = <ButtonFlat> {
                    margin:{right: 2}
                    draw_bg: {
                        border_size: 0.0
                    }
                    icon_walk: {
                        width: 12, height: Fit,
                        margin: { left: 10 }
                    }
                    draw_icon: {
                        color: #x0
                        color_hover: #3c3c3c
                        color_down: #x0
                        color_focus: #x0
                        svg_file: dep("crate://self/resources/copy.svg"),
                    }
                }
            }
            code_view = <CodeView>{
                editor: {
                    margin: {top: -2, bottom: 2}
                    pad_left_top: vec2(10.0,10.0)
                    width: Fill,
                    height: Fit,
                    draw_bg: { color: #1d2330 },
                    draw_text: {
                        text_style: {
                            font_size: 10,
                        }
                    }

                    // Inspired by Electron Highlighter theme https://electron-highlighter.github.io
                    token_colors: {
                        whitespace: #a8b5d1,        // General text/punctuation color as fallback
                        delimiter: #a8b5d1,          // punctuation
                        delimiter_highlight: #c5cee0, // Using a slightly brighter gray for highlight
                        error_decoration: #f44747,   // token.error-token
                        warning_decoration: #cd9731, // token.warn-token
                        
                        unknown: #a8b5d1,          // General text color
                        branch_keyword: #d2a6ef,     // keyword.control
                        constant: #ffd9af,         // constant.numeric
                        identifier: #a8b5d1,         // variable
                        loop_keyword: #d2a6ef,       // keyword.control.loop
                        number: #ffd9af,           // constant.numeric
                        other_keyword: #d2a6ef,      // keyword
                        punctuator: #a8b5d1,         // punctuation
                        string: #58ffc7,           // string
                        function: #82aaff,         // entity.name.function
                        typename: #fcf9c3,         // entity.name.class/type
                        comment: #506686,          // comment
                    }
                }
            }
        }
        use_code_block_widget: true,
        list_item_layout: { padding: {left: 10.0, right:10, top: 6.0, bottom: 0}, }
        list_item_walk:{margin:0, height:Fit, width:Fill}
        code_layout: { padding: {top: 10.0, bottom: 10.0}}
        quote_layout: { padding: {top: 10.0, bottom: 10.0}}

        link = {
            padding: { top: 1, bottom: 0 },
            draw_text: {
                color: #00f,
                color_pressed: #f00,
                color_hover: #0f0,
            }
        }

        draw_normal: {
            color: (MD_FONT_COLOR),
            text_style: {
                line_spacing: (MD_LINE_SPACING)
            }
        }
        draw_italic: {
            color: (MD_FONT_COLOR),
            text_style: {
                line_spacing: (MD_LINE_SPACING)
            }
        }
        draw_bold: {
            color: (MD_FONT_COLOR),
            text_style: {
                line_spacing: (MD_LINE_SPACING)
            }
        }
        draw_bold_italic: {
            color: (MD_FONT_COLOR),
            text_style: {
                line_spacing: (MD_LINE_SPACING)
            }
        }
        draw_fixed: {
            color: (MD_FONT_COLOR),
            text_style: {
                line_spacing: (MD_LINE_SPACING)
            }
        }
        draw_block: {
            line_color: (MD_FONT_COLOR)
            sep_color: (#EDEDED)
            quote_bg_color: (#EDEDED)
            quote_fg_color: (#969696)
            code_color: (#EDEDED)
        }
    }
}
