use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::shared::modal::*;

    use crate::mcp::mcp_servers::McpServers;

    pub McpScreen = {{McpScreen}} {
        width: Fill, height: Fill
        spacing: 20
        flow: Down

        header = <View> {
            height: Fit
            spacing: 20
            flow: Down

            padding: {left: 30, top: 40}
            <Label> {
                draw_text:{
                    text_style: <BOLD_FONT>{font_size: 25}
                    color: #000
                }
                text: "MCP Servers"
            }

            <Label> {
                draw_text:{
                    text_style: <BOLD_FONT>{font_size: 12}
                    color: #000
                }
                text: "Manage MCP servers and tools"
            }
        }

        mcp_servers = <McpServers> {}
    }
}

#[derive(Widget, LiveHook, Live)]
pub struct McpScreen {
    #[deref]
    view: View,
}

impl Widget for McpScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for McpScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let stack_navigation = self.stack_navigation(ids!(navigation));
        stack_navigation.handle_stack_view_actions(cx, actions);
    }
}
