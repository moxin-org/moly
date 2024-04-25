use makepad_widgets::*;
use moxin_protocol::data::DownloadedFile;

use crate::{
    data::store::Store,
    shared::utils::{open_folder, BYTES_PER_MB},
};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import makepad_draw::shader::std::*;

    import crate::shared::styles::*;

    import crate::my_models::downloaded_files_table::DownloadedFilesTable;

    ICON_EDIT_FOLDER = dep("crate://self/resources/icons/edit_folder.svg")
    ICON_SEARCH = dep("crate://self/resources/icons/search.svg")
    ICON_SHOW_IN_FILES = dep("crate://self/resources/icons/visibility.svg")

    DownloadLocation = <RoundedView> {
        width: Fit,
        height: Fit,
        padding: {top: 6, bottom: 6, left: 4, right: 14}
        align: {y: 0.5}
        spacing: 8,

        draw_bg: {
            instance radius: 2.0,
            color: #FEFEFE,
        }

        <Icon> {
            draw_icon: {
                svg_file: (ICON_EDIT_FOLDER),
                fn get_color(self) -> vec4 {
                    return #000;
                }
            }
            icon_walk: {width: 14, height: Fit}
        }

        <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 11},
                color: #000
            }
            text: "Change Download Location"
        }
    }

    ShowInFiles = <RoundedView> {
        width: Fit,
        height: Fit,
        margin: {left: 10}
        padding: {top: 6, bottom: 6, left: 4, right: 10}
        spacing: 8,
        cursor: Hand
        align: {y: 0.5}

        draw_bg: {
            instance radius: 2.0,
            color: #FEFEFE,
        }

        <Icon> {
            draw_icon: {
                svg_file: (ICON_SHOW_IN_FILES),
                fn get_color(self) -> vec4 {
                    return #000;
                }
            }
            icon_walk: {width: 14, height: Fit}
        }

        label = <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 11},
                color: #000
            }
            text: "Show in finder"
        }
    }

    SearchBar = <RoundedView> {
        width: Fit,
        height: Fit,

        show_bg: true,
        draw_bg: {
            color: #fff
        }

        padding: {top: 3, bottom: 3, left: 20, right: 20}

        spacing: 4,
        align: {x: 0.0, y: 0.5},

        draw_bg: {
            radius: 10.0,
            border_color: #D0D5DD,
            border_width: 1.0,
        }

        <Icon> {
            draw_icon: {
                svg_file: (ICON_SEARCH),
                fn get_color(self) -> vec4 {
                    return #666;
                }
            }
            icon_walk: {width: 14, height: Fit}
        }

        input = <TextInput> {
            width: 260,
            height: Fit,

            empty_message: "Search Model by Keyword"
            draw_bg: {
                color: #fff
            }
            draw_text: {
                text_style:<REGULAR_FONT>{font_size: 11},
                fn get_color(self) -> vec4 {
                    return #555
                }
            }

            // TODO find a way to override colors
            draw_cursor: {
                instance focus: 0.0
                uniform border_radius: 0.5
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(
                        0.,
                        0.,
                        self.rect_size.x,
                        self.rect_size.y,
                        self.border_radius
                    )
                    sdf.fill(mix(#fff, #bbb, self.focus));
                    return sdf.result
                }
            }

            // TODO find a way to override colors
            draw_select: {
                instance hover: 0.0
                instance focus: 0.0
                uniform border_radius: 2.0
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(
                        0.,
                        0.,
                        self.rect_size.x,
                        self.rect_size.y,
                        self.border_radius
                    )
                    sdf.fill(mix(#eee, #ddd, self.focus)); // Pad color
                    return sdf.result
                }
            }
        }
    }

    MyModelsScreen = {{MyModelsScreen}} {
        width: Fill
        height: Fill
        padding: 60
        spacing: 20
        flow: Down

        show_bg: true
        draw_bg: {
            color: #cccccc33,
            instance color2: #AF56DA55
            fn get_color(self) -> vec4 {
                let coef = self.rect_size.y / self.rect_size.x;

                let distance_vec = self.pos - vec2(0.8, 0.8);
                let norm_distance = length(vec2(distance_vec.x, distance_vec.y * coef) * 1.8);

                if pow(norm_distance, 1.4) > 1.0 {
                    return self.color;
                } else {
                    return mix(self.color2, self.color, pow(norm_distance, 1.4));
                }
            }

            fn pixel(self) -> vec4 {
                return Pal::premul(self.get_color());
            }
        }

        header = <View> {
            width: Fill, height: Fit
            spacing: 15
            flow: Right
            align: {x: 0.0, y: 1.0}

            title = <Label> {
                draw_text:{
                    text_style: <BOLD_FONT>{font_size: 30}
                    color: #000
                }
                text: "My Models"
            }

            models_summary = <Label> {
                draw_text:{
                    text_style: <REGULAR_FONT>{font_size: 20}
                    color: #555
                }
            }
        }

        sub_header = <View> {
            width: Fill, height: Fit
            flow: Right
            spacing: 10
            margin: {top: 10}
            align: {x: 0.0, y: 0.5}

            <DownloadLocation> {}
            show_in_files = <ShowInFiles> {}
            <View> { width: Fill, height: Fit }
            <SearchBar> {}
        }

        table = <DownloadedFilesTable> {
            margin: {top: 20}
        }
    }
}

#[derive(Widget, LiveHook, Live)]
pub struct MyModelsScreen {
    #[deref]
    view: View,
}

impl Widget for MyModelsScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.match_event(cx, event);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let downloaded_files = &scope.data.get::<Store>().unwrap().downloaded_files;

        let summary = generate_models_summary(&downloaded_files);
        let models_summary_label = self.view.label(id!(header.models_summary));
        models_summary_label.set_text(&summary);

        self.view
            .label(id!(show_in_files.label))
            .set_text(&file_manager_label());

        self.view.draw_walk(cx, scope, walk)
    }
}

fn file_manager_label() -> String {
    if cfg!(target_os = "windows") {
        "Show in Explorer".to_string()
    } else if cfg!(target_os = "macos") {
        "Show in Finder".to_string()
    } else {
        "Show in File Manager".to_string()
    }
}

impl MatchEvent for MyModelsScreen {
    fn handle_actions(&mut self, _cx: &mut Cx, actions: &Actions) {
        if let Some(fe) = self.view(id!(show_in_files)).finger_up(actions) {
            if fe.was_tap() {
                // TODO: replace with actual downloads path in the current store.
                open_folder(".").expect("Failed to open downloads folder");
            }
        }
    }
}

fn generate_models_summary(downloaded_files: &Vec<DownloadedFile>) -> String {
    let total_diskspace_mb = total_files_disk_space(downloaded_files);
    let disk_space_label = if total_diskspace_mb >= 1024.0 {
        format!("{:.2} GB Diskspace", total_diskspace_mb / 1024.0)
    } else {
        format!("{} MB Diskspace", total_diskspace_mb as i32)
    };

    let model_label = if downloaded_files.len() == 1 {
        "Model"
    } else {
        "Models"
    };

    format!(
        "{} {}, {}",
        downloaded_files.len(),
        model_label,
        disk_space_label
    )
}

fn total_files_disk_space(files: &Vec<DownloadedFile>) -> f64 {
    files.iter().fold(0., |acc, file| {
        let file_size_bytes = file.file.size.parse::<f64>();
        match file_size_bytes {
            Ok(size_bytes) => acc + (size_bytes / BYTES_PER_MB),
            Err(_) => acc,
        }
    })
}
