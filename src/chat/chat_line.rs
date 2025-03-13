use crate::chat::chat_line_loading::ChatLineLoadingWidgetExt;
use crate::chat::shared::ChatAgentAvatarWidgetExt;
use crate::data::providers::{Article, RemoteModel};
use makepad_widgets::markdown::MarkdownWidgetExt;
use makepad_widgets::*;

use reqwest::header::{HeaderValue, REFERER, USER_AGENT};
use url::Url;
use url_preview::{PreviewService, Preview, PreviewError};

use std::collections::{HashMap, HashSet};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use makepad_code_editor::code_view::CodeView;
    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::shared::resource_imports::*;
    use crate::chat::chat_line_loading::ChatLineLoading;
    use crate::chat::shared::ChatModelAvatar;
    use crate::chat::shared::ChatAgentAvatar;

    ICON_EDIT = dep("crate://self/resources/icons/edit.svg")
    ICON_DELETE = dep("crate://self/resources/icons/delete.svg")
    ICON_EXTERNAL_LINK = dep("crate://self/resources/icons/external_link.svg")

    ChatLineEditButton = <MolyButton> {
        width: 56,
        height: 31,
        spacing: 6,

        draw_bg: { color: #099250 }

        draw_text: {
            text_style: <REGULAR_FONT>{font_size: 9},
            fn get_color(self) -> vec4 {
                return #fff;
            }
        }
    }

    SaveButton = <ChatLineEditButton> {
        text: "Save"
    }

    SaveAndRegerateButton = <ChatLineEditButton> {
        width: 130,
        text: "Save & Regenerate"
    }

    CancelButton = <ChatLineEditButton> {
        draw_bg: { border_color: #D0D5DD, border_width: 1.0, color: #fff }

        draw_text: {
            text_style: <REGULAR_FONT>{font_size: 9},
            fn get_color(self) -> vec4 {
                return #000;
            }
        }
        text: "Cancel"
    }

    ArticlesList = {{ArticlesList}} {
        margin: {top: 15}
        height: Fit, width: Fill,
        flow: RightWrap, spacing: 10
        article_template: {{LinkPreviewUI}}<RoundedView> {
            cursor: Hand,
            height: 50, width: 180
            flow: Right, spacing: 10
            show_bg: true,
            draw_bg: {
                color: #f2f2f2
                radius: 3
            }

            padding: {left: 8, right: 8, top: 4, bottom: 4}
            align: {y: 0.5, x: 0.0}
            image_wrapper = <View> {
                align: {y: 0.5, x: 0.5},
                width: Fit, height: Fill,
                visible: true,
                external_link_icon = <Icon> {
                    draw_icon: {
                        svg_file: (ICON_EXTERNAL_LINK),
                        fn get_color(self) -> vec4 {
                            return #x0;
                        }
                    }
                    icon_walk: {width: 16, height: 16}
                }
            }
            flow_down_wrapper = <View> {
                flow: Down, spacing: 5
                align: {y: 0.5, x: 0.0}
                title = <Label> {
                    text: "Loading..."
                    draw_text: {
                        text_style: <BOLD_FONT>{font_size: 8},
                        color: #000
                    }
                }
                domain = <Label> {
                    text: "Loading..."
                    draw_text: {
                        color: #000,
                        text_style: <REGULAR_FONT>{font_size: 7}
                    }
                }
            }
        }
    }

    MessageText = <Markdown> {
        padding: 0,
        // Workaround: we should be using `paragraph_spacing: 20`,
        // but this property causes an unintended initial space so let's disable it.
        paragraph_spacing: 0
        font_color: #000,
        width: Fill, height: Fit,
        font_size: 10.0,
        code_block = <View> {
            width:Fill,
            height:Fit,
            code_view = <CodeView>{
                editor: {
                    pad_left_top: vec2(10.0,10.0)
                    width: Fill,
                    height: Fit,
                    draw_bg: { color: #3c3c3c },
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
    }

    EditTextInput = <MolyTextInput> {
        width: Fill,
        height: Fit,
        padding: 20,
        empty_message: ""

        draw_bg: {
            color: #fff,
            border_width: 1.0
            border_color: #D0D5DD
        }

        draw_text: {
            text_style:<REGULAR_FONT>{font_size: 10},
            word: Wrap,

            instance prompt_enabled: 0.0
            fn get_color(self) -> vec4 {
                return #000;
            }
        }
    }

    ChatLineBody = <View> {
        width: Fill,
        height: Fit,
        spacing: 20,
        flow: Down,

        sender_name_layout = <View> {
            height: 20,
            align: {x: 0.0, y: 0.85},

            sender_name = <Label> {
                width: Fit,
                height: Fit,
                draw_text:{
                    text_style: <BOLD_FONT>{font_size: 10},
                    color: #000
                }
            }
        }

        bubble = <RoundedView> {
            show_bg: true,
            draw_bg: {
                radius: 12.0,
            },

            width: Fill,
            height: Fit,
            flow: Down,
            padding: {left: 16, right: 18, top: 18, bottom: 14},
            align: {x: 0.5, y: 0.0},

            input_container = <View> {
                visible: false,
                width: Fill,
                height: Fit,
                input = <EditTextInput> {
                }
            }

            loading_container = <View> {
                width: Fill,
                height: Fit,
                loading = <ChatLineLoading> {}
            }

            markdown_message_container = <View> {
                width: Fill,
                height: Fit,
                markdown_message = <MessageText> {}
            }

            plain_text_message_container = <View> {
                width: Fill,
                height: Fit,
                plain_text_message = <Label> {
                    width: Fill,
                    height: Fit,
                    draw_text: {
                        text_style: <REGULAR_FONT>{height_factor: (1.3*1.3), font_size: 10},
                        color: #000
                    }
                }
            }

            articles_container = <View> {
                visible: false,
                width: Fill,
                height: Fit,
                articles = <ArticlesList> {}
            }

            edit_buttons = <View> {
                visible: false,
                width: Fit,
                height: Fit,
                margin: {top: 10},
                spacing: 6,
                save = <SaveButton> {}
                save_and_regenerate = <SaveAndRegerateButton> {}
                cancel = <CancelButton> {}
            }
        }
    }

    ChatLineActionButton = <MolyButton> {
        width: 14
        height: 14
        draw_icon: {
            color: #BDBDBD
            color_hover: #000
        }
        padding: 0,
        icon_walk: {width: 14, height: 14}
        draw_bg: {
            color: #0000
            color_hover: #0000
            border_width: 0
        }
        text: ""
    }

    pub ChatLine = {{ChatLine}} {
        padding: {top: 10, bottom: 3},
        width: Fill,
        height: Fit,

        avatar_section = <View> {
            width: Fit,
            height: Fit,
            margin: {left: 20, right: 12},

            model = <ChatModelAvatar> {}
            agent = <ChatAgentAvatar> { visible: false }
        }

        main_section = <View> {
            width: Fill,
            height: Fit,

            flow: Down,
            spacing: 8,

            body_section = <ChatLineBody> {}

            actions_section = <View> {
                width: Fill,
                height: 16,
                actions = <View> {
                    width: Fill,
                    height: Fit,
                    visible: false,
                    spacing: 6,

                    copy_button = <ChatLineActionButton> {
                        draw_icon: { svg_file: (ICON_COPY) }
                    }
                    edit_button = <ChatLineActionButton> {
                        draw_icon: { svg_file: (ICON_EDIT) }
                    }
                    delete_button = <ChatLineActionButton> {
                        draw_icon: { svg_file: (ICON_DELETE) }
                    }
                }
            }
        }

    }
}

#[derive(Clone, DefaultNone, Debug)]
pub enum ChatLineAction {
    Delete(usize),
    Edit(usize, String, bool),
    None,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum ChatLineState {
    #[default]
    Editable,
    NotEditable,
    OnEdit,
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatLine {
    #[deref]
    view: View,

    #[rust]
    message_id: usize,

    #[rust]
    edition_state: ChatLineState,

    #[rust]
    hovered: bool,
}

impl Widget for ChatLine {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        // Current Makepad's processing of the hover events is not enough
        // in our case because it collapes the hover state of the
        // children widgets (specially, the text input widget). So, we rely
        // on this basic mouse over calculation to show the actions buttons.
        if matches!(self.edition_state, ChatLineState::Editable) {
            if let Event::MouseMove(e) = event {
                let hovered = self.view.area().rect(cx).contains(e.abs);
                if self.hovered != hovered {
                    self.hovered = hovered;
                    self.view(id!(actions_section.actions))
                        .set_visible(cx, hovered);
                    self.redraw(cx);
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ChatLine {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        match self.edition_state {
            ChatLineState::Editable => self.handle_editable_actions(cx, actions),
            ChatLineState::OnEdit => self.handle_on_edit_actions(cx, actions),
            ChatLineState::NotEditable => {}
        }
    }
}

impl ChatLine {
    pub fn set_edit_mode(&mut self, cx: &mut Cx, enabled: bool) {
        self.edition_state = if enabled {
            ChatLineState::OnEdit
        } else {
            ChatLineState::Editable
        };

        self.view(id!(actions_section.actions))
            .set_visible(cx, false);
        self.view(id!(edit_buttons)).set_visible(cx, enabled);
        self.view(id!(input_container)).set_visible(cx, enabled);
        self.show_or_hide_message_label(cx, !enabled);

        self.redraw(cx);
    }

    pub fn show_or_hide_message_label(&mut self, cx: &mut Cx, show: bool) {
        // let text = self.text_input(id!(input)).text();
        // let to_markdown = parse_markdown(&text);
        // let is_plain_text = to_markdown.nodes.len() <= 3;
        // Temporary workaround to always show markdown.
        // This will be replaced by MolyKit.
        let is_plain_text = false;
        self.view(id!(plain_text_message_container))
            .set_visible(cx, show && is_plain_text);
        self.view(id!(markdown_message_container))
            .set_visible(cx, show && !is_plain_text);
    }

    pub fn handle_editable_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.button(id!(delete_button)).clicked(&actions) {
            cx.action(ChatLineAction::Delete(self.message_id));
        }

        if self.button(id!(edit_button)).clicked(&actions) {
            self.set_edit_mode(cx, true);
        }

        if self.button(id!(copy_button)).clicked(&actions) {
            let text_to_copy = self.text_input(id!(input)).text();
            cx.copy_to_clipboard(&text_to_copy);
        }
    }

    pub fn handle_on_edit_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.button(id!(save)).clicked(&actions) {
            let updated_message = self.text_input(id!(input)).text();

            // Do not allow to have empty messages for now.
            // TODO We should disable Save button when the message is empty.
            if !updated_message.trim().is_empty() {
                cx.action(ChatLineAction::Edit(
                    self.message_id,
                    updated_message,
                    false,
                ));
            }

            self.set_edit_mode(cx, false);
        }

        if let Some(val) = self.text_input(id!(input)).returned(actions) {
            if !val.trim().is_empty() {
                cx.action(ChatLineAction::Edit(self.message_id, val, false));
            }

            self.set_edit_mode(cx, false);
        }

        if self.button(id!(save_and_regenerate)).clicked(&actions) {
            let updated_message = self.text_input(id!(input)).text();

            // TODO We should disable Save and Regenerate button when the message is empty.
            if !updated_message.trim().is_empty() {
                cx.action(ChatLineAction::Edit(self.message_id, updated_message, true));
            }

            self.set_edit_mode(cx, false);
        }

        if self.button(id!(cancel)).clicked(&actions) {
            self.set_edit_mode(cx, false);
        }
    }
}

impl ChatLineRef {
    pub fn set_sender_name(&mut self, cx: &mut Cx, text: &str) {
        let Some(inner) = self.borrow_mut() else {
            return;
        };
        inner.label(id!(sender_name)).set_text(cx, text);
    }

    pub fn set_model_avatar_text(&mut self, cx: &mut Cx, text: &str) {
        let Some(inner) = self.borrow_mut() else {
            return;
        };
        inner.view(id!(avatar_section.model)).set_visible(cx, true);
        inner
            .chat_agent_avatar(id!(avatar_section.agent))
            .set_visible(false);
        inner.label(id!(avatar_label)).set_text(cx, text);
    }

    pub fn set_model_avatar(&mut self, cx: &mut Cx, model: &RemoteModel) {
        let Some(inner) = self.borrow_mut() else {
            return;
        };
        inner.view(id!(avatar_section.model)).set_visible(cx, false);
        inner.chat_agent_avatar(id!(avatar_section.agent)).set_visible(true);
        inner.chat_agent_avatar(id!(avatar_section.agent)).set_agent(model);
    }

    pub fn set_message_content(&mut self, cx: &mut Cx, text: &str, articles: &Vec<Article>, is_streaming: bool) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };

        match inner.edition_state {
            ChatLineState::Editable | ChatLineState::NotEditable => {
                if is_streaming && !text.is_empty() {
                    let output = format!("{}{}", text, "●");
                    inner.text_input(id!(input)).set_text(cx, &output.trim());
                    inner
                        .label(id!(plain_text_message))
                        .set_text(cx, &output.trim());
                    inner
                        .markdown(id!(markdown_message))
                        .set_text(cx, &output.trim());
                } else {
                    inner.text_input(id!(input)).set_text(cx, text.trim());
                    inner
                        .label(id!(plain_text_message))
                        .set_text(cx, text.trim());
                    inner
                        .markdown(id!(markdown_message))
                        .set_text(cx, &text.trim().replace("\n\n", "\n\n\u{00A0}\n\n"));
                }

                if !articles.is_empty() {
                    inner.view(id!(articles_container)).set_visible(cx, true);
                    let mut articles_ref = inner.articles_list(id!(articles));
                    articles_ref.set_articles(cx, &articles);
                } else {
                    inner.view(id!(articles_container)).set_visible(cx, false);
                }

                // We know only AI assistant messages could be empty, so it is never
                // displayed in user's chat lines.
                let show_loading = text.trim().is_empty();
                inner
                    .view(id!(loading_container))
                    .set_visible(cx, show_loading);

                let mut loading_widget = inner.chat_line_loading(id!(loading_container.loading));
                if show_loading {
                    loading_widget.animate(cx);
                } else {
                    loading_widget.stop_animation();
                }

                inner.show_or_hide_message_label(cx, true);
            }
            ChatLineState::OnEdit => {}
        }
    }

    pub fn set_message_id(&mut self, message_id: usize) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.message_id = message_id;
    }

    pub fn set_actions_enabled(&mut self, cx: &mut Cx, enabled: bool) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };

        if enabled {
            if inner.edition_state == ChatLineState::NotEditable {
                inner.edition_state = ChatLineState::Editable;
            }
        } else {
            inner.edition_state = ChatLineState::NotEditable;
            inner
                .view(id!(actions_section.actions))
                .set_visible(cx, false);
        }
    }

    pub fn set_regenerate_button_visible(&mut self, cx: &mut Cx, visible: bool) {
        let Some(inner) = self.borrow_mut() else {
            return;
        };
        inner
            .button(id!(save_and_regenerate))
            .set_visible(cx, visible);
    }
}



#[derive(Live, LiveHook, Widget)]
pub struct ArticlesList {
    #[deref]
    view: View,

    /// The template for the citation views.
    #[live]
    article_template: Option<LivePtr>,

    /// The views that represent the citations.
    #[rust]
    link_preview_children: ComponentMap<usize, LinkPreviewUI>,

    /// The citations (URLs) that are currently being rendered.
    #[rust]
    articles: Vec<Article>,

    /// Maps the index of the citation to the link preview.
    #[rust]
    link_previews: HashMap<usize, Preview>,

    /// Maps the index of the citation to the image blob.
    #[rust]
    image_blobs: HashMap<usize, Vec<u8>>,

    /// Track which images have already been loaded
    #[rust]
    loaded_image_indices: HashSet<usize>,
}

impl Widget for ArticlesList {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        // self.ui_runner().handle(cx, event, scope, self);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // TODO: Fix this, currently redrawing on every event
        // And at the same time, the citations are not being redrawn unless there's a user-triggered event like mouse move or window resize.
        cx.begin_turtle(walk, self.layout);
        for (link_preview_id, link_preview_ui) in self.link_preview_children.iter_mut() {
            let article_content = &self.articles[*link_preview_id];
            let domain = url::Url::parse(&article_content.url)
                .ok()
                .and_then(|u| u.domain().map(|d| d.to_string()))
                .unwrap_or_default();
            link_preview_ui.url = article_content.url.clone();
            link_preview_ui.label(id!(domain)).set_text(cx, &domain);
            link_preview_ui.label(id!(title)).set_text(cx, &article_content.title);

            if let Some(link_preview) = self.link_previews.get(link_preview_id) {
                if let Some(title) = &link_preview.title {
                    link_preview_ui.label(id!(title)).set_text(cx, &title);
                }
                if let Some(image_url) = &link_preview.image_url {
                    if let Some(image_bytes) = self.image_blobs.get(link_preview_id) {
                        // Only load image if it's not already loaded
                        if !self.loaded_image_indices.contains(link_preview_id) {
                            if is_jpeg(image_bytes) {
                                let _ = link_preview_ui.image(id!(image)).load_jpg_from_data(cx, &image_bytes);
                                link_preview_ui.image(id!(image)).apply_over(cx,
                                live! {
                                    width: 75, height: 75
                                });
                            } else if is_png(image_bytes) {
                                link_preview_ui.image(id!(image)).apply_over(cx,
                                live! {
                                    width: 75, height: 75
                                });
                                let _ = link_preview_ui.image(id!(image)).load_png_from_data(cx, &image_bytes);
                            } else {
                                // TODO: handle other image types
                                // Do not try again
                                self.loaded_image_indices.insert(*link_preview_id);
                            }
                            self.loaded_image_indices.insert(*link_preview_id);
                        }
                    }
                }
            }
            link_preview_ui.draw(cx, scope);
        }
        cx.end_turtle();
        DrawStep::done()
    }
}

impl ArticlesList {
    fn save_link_preview(&mut self, cx: &mut Cx, index: usize, link_preview: Preview) {
        // let image_url = link_preview.image_url.clone();
        // self.link_previews.insert(index, link_preview);
        // // Only fetch if we don't already have this image
        // if let Some(image_url) = image_url {
        //     if !self.image_blobs.contains_key(&index) {
        //         let ui = self.ui_runner();
        //         spawn(async move {
        //             let fetched_image = fetch_image_blob(&image_url).await;
        //             if let Ok(image_bytes) = fetched_image {
        //                 ui.defer_with_redraw(move |me, cx, _scope| {
        //                     me.image_blobs.insert(index, image_bytes);
        //                 });
        //             }
        //         });
        //     }
        // }
    }

    fn update_articles(&mut self, cx: &mut Cx, articles: &Vec<Article>) {
        self.visible = true;
        // compare the vecs, if they are the same, return
        if self.articles.len() == articles.len() {
            let is_same = self.articles.iter().zip(articles.iter())
                .all(|(a, b)| a == b);
            if is_same {
                return;
            }
        }

        self.articles = articles.clone();
        self.visible = true;
        self.link_preview_children.clear();
        self.loaded_image_indices.clear();
        self.image_blobs.clear();

        for (index, article) in articles.iter().enumerate() {
            let new_article = LinkPreviewUI::new_from_ptr(cx, self.article_template);
            self.link_preview_children.insert(index, new_article);

            let article_clone = article.clone();
            let index_clone = index;
            let ui = self.ui_runner();
            let widget_uid = self.widget_uid();

            // TODO: rework this to use caching and batch fetching from the url-preview crate.
            // spawn(async move {
            //     let future = async {
            //         let preview = PreviewService::new().generate_preview(&article_clone).await;
            //         match preview {
            //             Ok(preview) => {
            //                 ui.defer_with_redraw(move |me, cx, _scope| {
            //                     me.save_link_preview(cx, index_clone, preview);
            //                 });
            //             }
            //             Err(e) => {
            //                 eprintln!("Error fetching preview for index {}: {:?}", index_clone, e);
            //             }
            //         }
            //     };

            //     let (future, _abort_handle) = futures::future::abortable(future);
            //     future.await.unwrap_or_else(|_| eprintln!("Preview fetch aborted for index {}", index_clone));
            // });
        }

        self.redraw(cx);
    }
}

impl ArticlesListRef {
    fn set_articles(&mut self, cx: &mut Cx, articles: &Vec<Article>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.update_articles(cx, articles);
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct LinkPreviewUI {
    #[deref]
    view: View,

    #[rust]
    url: String,
}

impl Widget for LinkPreviewUI {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for LinkPreviewUI {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        // TODO: these finger up events are not reaching here.
        if let Some(item) = actions.find_widget_action(self.widget_uid()) {
            if let ViewAction::FingerUp(fd) = item.cast() {
                let _ = robius_open::Uri::new(&self.url).open();
            }
        }
    }
}

async fn fetch_image_blob(url: &str) -> Result<Vec<u8>, reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        // Trick the server into thinking we're a browser
        .header(USER_AGENT, HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36"
        ))
        .send()
        .await?;

    let bytes = response.bytes().await?;
    Ok(bytes.to_vec())
}

fn is_jpeg(bytes: &[u8]) -> bool {
    bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xD8
}

fn is_png(bytes: &[u8]) -> bool {
    bytes.len() >= 4 
        && bytes[0] == 0x89 
        && bytes[1] == 0x50 
        && bytes[2] == 0x4E 
        && bytes[3] == 0x47
}
