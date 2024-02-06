use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import makepad_draw::shader::std::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::shared::icon::Icon;

    const MODEL_LINK_FONT_COLOR = #x155EEF
    ICON_INFO = dep("crate://self/resources/icons/info.svg")

    ModelLink = <LinkLabel> {
        width: Fill,
        draw_text: {
            text_style: <REGULAR_FONT>{font_size: 10},
            fn get_color(self) -> vec4 {
                return mix(
                    mix(
                        MODEL_LINK_FONT_COLOR,
                        MODEL_LINK_FONT_COLOR,
                        self.hover
                    ),
                    MODEL_LINK_FONT_COLOR,
                    self.pressed
                )
            }
        }
    }

    ModelAttributeTag = <RoundedView> {
        spacing: 5,
        draw_bg: {
            instance radius: 2.0,
        }

        attr_name = <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 10},
                color: #fff
            }
        }

        attr_value = <Label> {
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 10},
                color: #fff
            }
        }

    }

    ModelAttributes = <View> {
        width: Fit,
        height: Fit,
        spacing: 10,

        <ModelAttributeTag> {
            width: Fit,
            height: Fit,
            padding: 4,

            draw_bg: { color: #3538CD },
            attr_name = { text: "Model Size" }
            attr_value = { text: "7B params" }
        }

        <ModelAttributeTag> {
            width: Fit,
            height: Fit,
            padding: 4,

            draw_bg: { color: #CA8504 },
            attr_name = { text: "Requires" }
            attr_value = { text: "8GB+ RAM" }
        }

        <ModelAttributeTag> {
            width: Fit,
            height: Fit,
            padding: 4,

            draw_bg: { color: #FCCEEE },
            attr_name = {
                draw_text: { color: #C11574 },
                text: "Architecture"
            }
            attr_value = {
                draw_text: { color: #C11574 },
                text: "Mistral"
            }
        }
    }

    ModelHeading = <View> {
        height: Fit,
        <View> {
            width: Fit,
            height: Fit,
            flow: Down,
            spacing: 10,
            name = <Label> {
                draw_text:{
                    text_style: <BOLD_FONT>{font_size: 16},
                    color: #000
                }
                text: "OpenHermes 2.5 Mistral 7B"
            }
            <ModelAttributes> {}
        }
        <VerticalFiller> {}
        <View> {
            width: Fit,
            height: Fit,
            <ModelAttributeTag> {
                width: Fit,
                height: Fit,
                padding: 4,
    
                draw_bg: {
                    color: #0000,
                    border_color: #98A2B3,
                    border_width: 1.0,
                },
                attr_name = {
                    draw_text: { color: #000 }
                    text: "Released"
                }
                attr_value = {
                    draw_text: { color: #000 }
                    text: "Oct 29, 2023 (90 days ago)"
                }
            }
        }
    }

    ModelSummary = <View> {
        width: Fill,
        height: Fit,
        flow: Down,
        spacing: 20,

        <Label> {
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 12},
                color: #000
            }
            text: "Model Summary"
        }
        summary = <Label> {
            width: Fill,
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 10},
                word: Wrap,
                color: #000
            }
            text: "OpenHermes 2.5 Mistral 7B is an advanced iteration of the OpenHermes 2 language model, enhanced by training on a significant proportion of code datasets. This additional training improved performance across several benchmarks, notably TruthfulQA, AGIEval, and the GPT4All suite, while slightly decreasing the BigBench score. Notably, the model's ability to handle code-related tasks, measured by the humaneval score..."
        }
    }

    ModelDetails = <View> {
        width: 500,
        height: Fit,
        flow: Down,
        spacing: 20,

        <Label> {
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 12},
                color: #000
            }
            text: "Author"
        }

        <ModelLink> {
            width: Fill,
            text: "Teknium"
        }

        <Label> {
            width: Fill,
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 10},
                word: Wrap,
                color: #000
            }
            text: "Creator of numerous chart topping fine-tunes and a Co-founder of NousResearch"
        }

        <Label> {
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 12},
                color: #000
            }
            text: "Model Resources"
        }

        <ModelLink> {
            width: Fill,
            text: "Hugging Face"
        }
    }

    ModelInformation = <View> {
        width: Fill,
        height: Fit,
        spacing: 10,
        <ModelSummary> {}
        <ModelDetails> {}
    }

    ModelHighlightedFilesRow = <View> {
        width: Fill,
        height: Fit,

        show_bg: true,
        draw_bg: {
            color: #F9FAFB
        }

        cell1 = <View> { width: Fill, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
        cell2 = <View> { width: 250, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
        cell3 = <View> { width: 250, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
        cell4 = <View> { width: 200, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
    }

    ModelHighlightedFilesLabel = <RoundedView> {
        width: Fit,
        height: Fit,
        padding: 4,

        draw_bg: {
            instance radius: 2.0,
            color: #E6F4D7,
        }

        label = <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 10},
                color: #3F621A
            }
        }
    }

    ModelHighlightedFilesRowWithData = <ModelHighlightedFilesRow> {
        cell1 = {
            spacing: 10,
            filename = <Label> {
                draw_text:{
                    text_style: <BOLD_FONT>{font_size: 10},
                    color: #000
                }
            }
        }

        cell2 = {
            full_size = <Label> {
                draw_text:{
                    text_style: <REGULAR_FONT>{font_size: 10},
                    color: #000
                }
            }
        }

        cell3 = {
            quantization_tag = <RoundedView> {
                width: Fit,
                height: Fit,
                padding: 4,

                draw_bg: {
                    instance radius: 2.0,
                    color: #B9E6FE,
                }
        
                quantization = <Label> {
                    draw_text:{
                        text_style: <REGULAR_FONT>{font_size: 10},
                        color: #1849A9
                    }
                }

                <Icon> {
                    draw_icon: {
                        svg_file: (ICON_INFO), color: #00f,
                    }
                    icon_walk: {width: Fit, height: Fit}
                }
            }
        }

        cell4 = {
            align: {x: 0.5, y: 0.5},
            <Button> {
                text: "Download"
                width: 160,

                align: {x: 0.5, y: 0.5}

                draw_text: {
                    text_style: <REGULAR_FONT>{font_size: 10},
                    fn get_color(self) -> vec4 {
                        return mix(
                            mix(
                                #fff,
                                #fff,
                                self.hover
                            ),
                            #fff,
                            self.pressed
                        )
                    }
                }

                draw_bg: { bodytop: #099250, bodybottom: #099250  }
            }
        }
    }

    ModelHighlightedFiles = <View> {
        width: Fill,
        height: Fit,

        flow: Down

        <ModelHighlightedFilesRow> {
            cell1 = {
                <Label> {
                    draw_text:{
                        text_style: <BOLD_FONT>{font_size: 10},
                        color: #000
                    }
                    text: "Highlighted Files"
                }
            }

            cell4 = {
                align: {x: 0.5, y: 0.5},
                <ModelLink> {
                    width: Fill,
                    text: "See All Files"
                }
            }
        }

        <ModelHighlightedFilesRow> {
            show_bg: false,
            cell1 = {
                <Label> {
                    draw_text:{
                        text_style: <BOLD_FONT>{font_size: 10},
                        color: #667085
                    }
                    text: "Model File"
                }
            }

            cell2 = {
                <Label> {
                    draw_text:{
                        text_style: <BOLD_FONT>{font_size: 10},
                        color: #667085
                    }
                    text: "Full Size"
                }
            }

            cell3 = {
                <Label> {
                    draw_text:{
                        text_style: <BOLD_FONT>{font_size: 10},
                        color: #667085
                    }
                    text: "Quantization"
                }
            }
        }

        <ModelHighlightedFilesRowWithData> {
            cell1 = {
                filename = { text: "stablelm-zephyr-3b.Q4_K_S.gguf" }
                <ModelHighlightedFilesLabel> {
                    label = { text: "Small & Fast" }
                }
            }
            cell2 = { full_size = { text: "1.62 GB" }}
            cell3 = {
                quantization_tag = { quantization = { text: "Q4_K_S" }}
            }
        }
        <ModelHighlightedFilesRowWithData> {
            cell1 = {
                filename = { text: "stablelm-zephyr-3b.Q6_K.gguf" }
                <ModelHighlightedFilesLabel> {
                    label = { text: "Less Compressed" }
                }
                <ModelHighlightedFilesLabel> {
                    label = { text: "Might be slower" }
                }
            }
            cell2 = { full_size = { text: "2.30 GB" }}
            cell3 = {
                quantization_tag = { quantization = { text: "Q6_K" }}
            }
        }
    }

    ModelCard = <RoundedView> {
        width: Fill,
        height: Fit,

        draw_bg: {
            instance radius: 3.0,
            color: #F2F4F7
        }

        flow: Down,
        padding: 30,
        spacing: 30,

        <ModelHeading> {}
        <Line> {}
        <ModelInformation> {}
        <ModelHighlightedFiles> {}
    }
}