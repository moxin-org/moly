use makepad_widgets::*;
use moly_kit::widgets::moly_modal::MolyModalWidgetExt;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::chat::chat_history::ChatHistory;
    use crate::chat::chats_deck::ChatsDeck;
    use crate::settings::providers_screen::ProvidersScreen;
    use crate::settings::provider_view::ProviderView;
    use crate::mcp::mcp_screen::McpScreen;
    use moly_kit::widgets::moly_modal::*;

    ICON_NEW_CHAT = dep("crate://self/resources/icons/new_chat.svg")

    SettingsMenu = <MolyModal> {
        align: {x: 0.0, y: 0.0}
        bg_view: {
            visible: false
        }
        content: <RoundedView> {
            show_bg: true
            draw_bg: {border_radius: 5, color: #f}
            width: 150, height: Fit
            align: {x: 0.5, y: 0.5}
            flow: Down
            spacing: 10
            padding: {left: 10, right: 10, top: 20, bottom: 20}
            go_to_providers = <View> {
                align: {x: 0.5, y: 0.5}
                width: Fill, height: Fit
                cursor: Hand
                <Label> {
                    text: "Providers"
                    draw_text: {
                        color: #x0
                    }
                }
            }

            separator =<View> {
                width: Fill, height: 0.5
                show_bg: true
                draw_bg: {color: #d3d3d3}
                margin: {left: 10, right: 10}
            }

            go_to_mcp = <View> {
                align: {x: 0.5, y: 0.5}
                width: Fill, height: Fit
                cursor: Hand
                <Label> {
                    text: "MCP Servers"
                    draw_text: {
                        color: #x0
                    }
                }
            }
        }
    }

    HEADER_HEIGHT = 100
    MolyNavigationView = <StackNavigationView> {
        width: Fill, height: Fill
        draw_bg: { color: (MAIN_BG_COLOR) }
        header = {
            height: (HEADER_HEIGHT)
            content = {
                padding: {top: 10}
                align: {y: 0.5}
                button_container = {
                    align: {y: 0.5}
                    padding: {left: 16}
                    left_button = {
                        height: Fit,
                        icon_walk: {width: 12, height: Fit}
                        draw_icon: {
                            brightness: 0.0
                        }
                    }
                }
                title_container = {
                    title = {
                        draw_text: {
                            text_style: <BOLD_FONT>{font_size: 14},
                            color: #x0
                        }
                    }
                }
            }
        }
        body = { margin: {top: (HEADER_HEIGHT) }}
    }

    pub ChatScreenMobile = {{ChatScreenMobile}} {
        width: Fill, height: Fill
        flow: Overlay
        margin: { top: 40 }

        navigation = <StackNavigation> {
            width: Fill, height: Fill
            root_view = {
                width: Fill, height: Fill
                flow: Overlay
                menu_toggle = <View> {
                    margin: {top: 10, left: 20}
                    width: Fit, height: Fit
                    cursor: Hand
                    <IconSet> {
                        text: "" // FontAwesome f0c9
                        draw_text: {
                            color: #x0
                            text_style: { font_size: 18.0 }
                        }
                    }
                }

                <CachedWidget> {
                    chats_deck = <ChatsDeck> {}
                }
            }

            history_navigation_view = <MolyNavigationView> {
                header = {
                    content = {
                        title_container = {
                            title = {
                                text: "Chat History"
                            }
                        }
                        settings_button = <View> {
                            margin: {left: 100}
                            align: {x: 1.0, y: 0.5}
                            width: Fill, height: Fit
                            margin: {right: 15}
                            cursor: Hand
                            <IconSet> {
                                text: "" // FontAwesome f013
                                draw_text: {
                                    color: #333
                                    text_style: { font_size: 18.0 }
                                }
                            }
                        }
                        settings_menu = <SettingsMenu> {}
                    }
                }
                body = {
                    flow: Overlay
                    chat_history = <ChatHistory> {
                        width: Fill, height: Fill
                    }
                    align: { x: 0.95, y: 0.95 }
                    <RoundedView> {
                        show_bg: true
                        draw_bg: {
                            color: #x0
                            border_radius: 5.0
                        }
                        width: Fit, height: Fit

                        align: { x: 0.5, y: 0.5 }
                        new_chat_button = <MolyButton> {
                            width: Fit, height: Fit
                            padding: { left: 10, right: 10, top: 10, bottom: 10 }
                            icon_walk: {margin: { top: -1 }, width: 18, height: 18},
                            text: "New Chat",
                            draw_text: {
                                color: #f,
                                text_style: { font_size: 12.0 }
                            },
                            draw_icon: {
                                svg_file: (ICON_NEW_CHAT),
                                fn get_color(self) -> vec4 {
                                    return #f;
                                }
                            }
                        }
                    }
                }
            }

            mcp_navigation_view = <MolyNavigationView> {
                header = {
                    content = {
                        title_container = {
                            title = {
                                text: "MCP Servers"
                            }
                        }
                    }
                }
                body = {
                    mcp_screen = <McpScreen> {
                        width: Fill, height: Fill
                        header = { visible: false }
                    }
                }
            }

            providers_navigation_view = <MolyNavigationView> {
                header = {
                    content = {
                        title_container = {
                            title = {
                                text: "Providers"
                            }
                        }
                    }
                }
                body = {
                    providers = <ProvidersScreen> {
                        width: Fill, height: Fill
                        header = { visible: false }
                    }
                }
            }

            provider_navigation_view = <MolyNavigationView> {
                header = {
                    content = {
                        title_container = {
                            title = {
                                text: "Provider Settings"
                            }
                        }
                    }
                }
                body = {
                    padding: {top: 10}
                    provider_view = <ProviderView> {
                        width: Fill, height: Fill
                        padding: {left: 20, right: 20, top: 30, bottom: 30}
                    }
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatScreenMobile {
    #[deref]
    view: View,
}

impl Widget for ChatScreenMobile {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ChatScreenMobile {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let stack_navigation = self.stack_navigation(id!(navigation));
        stack_navigation.handle_stack_view_actions(cx, actions);

        // Menu Toggle
        if let Some(_evt) = self.view(id!(menu_toggle)).finger_down(actions) {
            stack_navigation.push(cx, live_id!(history_navigation_view));
        }

        let modal = self.moly_modal(id!(settings_menu));

        // Settings Menu
        if let Some(_evt) = self.view(id!(settings_button)).finger_down(actions) {
            // TODO: Ideally we should use the settings_button position but for some reason is always coming back fixed at 100.0
            let parent_view_width = self
                .stack_navigation_view(id!(history_navigation_view))
                .area()
                .rect(cx)
                .size
                .x;

            let button_rect = self.view(id!(settings_button)).area().rect(cx);
            let coords = dvec2(
                parent_view_width - 170.0,
                button_rect.pos.y + button_rect.size.y,
            );

            modal.apply_over(
                cx,
                live! {
                    content: { margin: { left: (coords.x), top: (coords.y) }}
                },
            );
            modal.open(cx);
        }

        // Go to Providers
        if let Some(_evt) = self.view(id!(go_to_providers)).finger_down(actions) {
            modal.close(cx);
            stack_navigation.push(cx, live_id!(providers_navigation_view));
        }

        // Go to MCP Servers
        if let Some(_evt) = self.view(id!(go_to_mcp)).finger_down(actions) {
            modal.close(cx);
            stack_navigation.push(cx, live_id!(mcp_navigation_view));
        }
    }
}
