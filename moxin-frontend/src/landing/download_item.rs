use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;

    ICON_PAUSE = dep("crate://self/resources/icons/pause_download.svg")
    ICON_CANCEL = dep("crate://self/resources/icons/cancel_download.svg")
    ICON_PLAY = dep("crate://self/resources/icons/play_download.svg")
    ICON_RETRY = dep("crate://self/resources/icons/retry_download.svg")

    ModelAttributeTag = <RoundedView> {
        width: Fit,
        height: Fit,
        padding: {top: 6, bottom: 6, left: 10, right: 10}

        spacing: 5,
        draw_bg: {
            radius: 2.0,
        }

        caption = <Label> {
            draw_text: {
                text_style: <REGULAR_FONT>{font_size: 9},
                color: #fff
            }
        }
    }

    Information = <View> {
        width: Fit,
        height: Fit,
        flow: Right,
        spacing: 12,
        margin: {right: 60}

        align: {x: 0.0, y: 0.5},

        architecture_tag = <ModelAttributeTag> {
            caption = {
                text: "StableLM"
            }
            draw_bg: {
                color: #A44EBB,
            }
        }

        params_size_tag = <ModelAttributeTag> {
            caption = {
                text: "3B"
            }
            draw_bg: {
                color: #44899A,
            }
        }

        filename = <Label> {
            draw_text: {
                text_style: <REGULAR_FONT>{font_size: 10},
                color: #000
            }
            text: "Stable-code-instruct-3b-Q8_0.gguf"
        }
    }

    Progress = <View> {
        width: 600,
        height: Fit,
        spacing: 8,

        flow: Down,

        <View> {
            width: Fill,
            height: Fit,

            flow: Right,

            <Label> {
                draw_text: {
                    text_style: <BOLD_FONT>{font_size: 9},
                    color: #099250
                }
                text: "Downloading 9.7%"
            }
            <View> { width: Fill, height: 1 }
            <Label> {
                draw_text: {
                    text_style: <REGULAR_FONT>{font_size: 9},
                    color: #667085
                }
                text: "288.55 MB / 2.97 GB | 10.59 MB/s "
            }
        }

        <View> {
            width: Fill,
            height: 12,

            flow: Overlay,

            <RoundedView> {
                width: 600,
                height: Fill,
                draw_bg: {
                    color: #D9D9D9,
                    radius: 2.0,
                }
            }

            <RoundedView> {
                width: 174,
                height: Fill,
                draw_bg: {
                    color: #099250,
                    radius: 2.0,
                }
            }
        }
    }

    ActionButton = <RoundedView> {
        width: 40,
        height: 40,

        align: {x: 0.5, y: 0.5}

        draw_bg: {
            border_color: #EAECF0,
            border_width: 1.0,
            color: #fff,
            radius: 2.0,
        }

        icon = <Icon> {
            draw_icon: {
                fn get_color(self) -> vec4 {
                    return #667085;
                }
            }
            icon_walk: {height: 12, margin: {top: 2, right: 4}}
        }
    }

    Actions = <View> {
        width: Fill,
        height: Fit,
        flow: Right,
        spacing: 12,

        align: {x: 0.5, y: 0.5},

        pause_button = <ActionButton> {
            icon = {
                draw_icon: {
                    svg_file: (ICON_PAUSE),
                }
            }

        }

        cancel_button = <ActionButton> {
            icon = {
                draw_icon: {
                    svg_file: (ICON_CANCEL),
                }
            }
        }
    }

    DownloadItem = <RoundedView> {
        width: Fill,
        height: Fit,

        padding: 20,
        margin: {bottom: 16},
        spacing: 30,
        align: {x: 0.0, y: 0.5},

        draw_bg: {
            border_color: #EAECF0,
            border_width: 1.0,
            color: #fff,
        }

        <Information> {}
        <Progress> {}
        <Actions> {}
    }
}
