use crate::widgets::{
    avatar::AvatarWidgetRefExt, slot::SlotWidgetRefExt,
    standard_message_content::StandardMessageContentWidgetRefExt,
};
use crate::{protocol::*, utils::makepad::events::EventExt};
use makepad_widgets::permission::Permission;
use makepad_widgets::permission::PermissionStatus;
use makepad_widgets::{makepad_platform::AudioDeviceType, *};
use std::sync::{Arc, Mutex};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::widgets::chat_lines::*;
    use crate::widgets::standard_message_content::*;

    AIAnimation = <RoundedView> {
        width: 200, height: 200
        show_bg: true
        // Shader based on "Branded AI assistant" by Vickone (https://www.shadertoy.com/view/tfcGD8)
        // Licensed under CC BY-NC-SA 3.0
        draw_bg: {
            // Simple hash function
            fn hash21(self, p: vec2) -> float {
                let mut p = fract(p * vec2(234.34, 435.345));
                p += dot(p, p + 34.23);
                return fract(p.x * p.y);
            }

            // Simple noise function
            fn noise(self, p: vec2) -> float {
                let i = floor(p);
                let f = fract(p);
                let f_smooth = f * f * (3.0 - 2.0 * f);
                let a = self.hash21(i);
                let b = self.hash21(i + vec2(1.0, 0.0));
                let c = self.hash21(i + vec2(0.0, 1.0));
                let d = self.hash21(i + vec2(1.0, 1.0));
                return mix(mix(a, b, f_smooth.x), mix(c, d, f_smooth.x), f_smooth.y);
            }

            // Simplified FBM (fractal brownian motion)
            fn fbm(self, p: vec2) -> float {
                let mut sum = 0.0;
                let mut amp = 0.5;
                let mut freq = 1.0;

                // Unroll the loop for compatibility
                sum += self.noise(p * freq) * amp;
                amp *= 0.5;
                freq *= 2.0;

                sum += self.noise(p * freq) * amp;
                amp *= 0.5;
                freq *= 2.0;

                sum += self.noise(p * freq) * amp;
                amp *= 0.5;
                freq *= 2.0;

                sum += self.noise(p * freq) * amp;
                amp *= 0.5;
                freq *= 2.0;

                return sum;
            }

            fn pixel(self) -> vec4 {
                // Center and aspect-correct UV coordinates
                let uv = (self.pos - 0.5) * 2.0;

                let mut col = vec3(0.1, 0.1, 0.1);
                // let mut col = vec3(0.0, 0.0, 0.0);

                let radius = 0.3 + sin(self.time * 0.5) * 0.02;
                let d = length(uv);

                let angle = atan(uv.y, uv.x);
                let wave = sin(angle * 3.0 + self.time) * 0.1;
                let wave2 = cos(angle * 5.0 - self.time * 1.3) * 0.08;

                let noise1 = self.fbm(uv * 3.0 + self.time * 0.1);
                let noise2 = self.fbm(uv * 5.0 - self.time * 0.2);

                let orb_color = vec3(0.2, 0.6, 1.0);
                let orb = smoothstep(radius + wave + wave2, radius - 0.1 + wave + wave2, d);

                let gradient1 = vec3(0.8, 0.2, 0.5) * sin(angle + self.time);
                let gradient2 = vec3(0.2, 0.5, 1.0) * cos(angle - self.time * 0.7);

                // Simplified particles (unrolled loop)
                let mut particles = 0.0;

                // Particle 1
                let particle_pos1 = vec2(
                    sin(self.time * 0.5) * 0.5,
                    cos(self.time * 0.3) * 0.5
                );
                particles += smoothstep(0.05, 0.0, length(uv - particle_pos1));

                // Particle 2
                let particle_pos2 = vec2(
                    sin(self.time * 0.7) * 0.5,
                    cos(self.time * 0.5) * 0.5
                );
                particles += smoothstep(0.05, 0.0, length(uv - particle_pos2));

                // Particle 3
                let particle_pos3 = vec2(
                    sin(self.time * 0.9) * 0.5,
                    cos(self.time * 0.7) * 0.5
                );
                particles += smoothstep(0.05, 0.0, length(uv - particle_pos3));

                // Combine all effects
                col += orb * mix(orb_color, gradient1, noise1);
                col += orb * mix(gradient2, orb_color, noise2) * 0.5;
                col += particles * vec3(0.5, 0.8, 1.0);
                col += exp(-d * 4.0) * vec3(0.2, 0.4, 0.8) * 0.5;

                // return vec4(col, 1.0);

                // Clip the final output to a circle
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                let radius = min(self.rect_size.x, self.rect_size.y) * 0.5;
                sdf.circle(
                    self.rect_size.x * 0.5,
                    self.rect_size.y * 0.5,
                    radius
                );

                sdf.fill_keep(vec4(col, 1.0));

                return sdf.result;
            }
        }
    }

    SimpleDropDown = <DropDown> {
        draw_text: {
            text_style: {font_size: 12}
            fn get_color(self) -> vec4 {
                return mix(
                    #2,
                    #x0,
                    self.down
                )
            }
        }

        popup_menu: {
            width: 300, height: Fit,
            flow: Down,
            padding: <THEME_MSPACE_1> {}

            menu_item: <PopupMenuItem> {
                width: Fill, height: Fit,
                align: { y: 0.5 }
                padding: {left: 15, right: 15, top: 10, bottom: 10}

                draw_text: {
                    fn get_color(self) -> vec4 {
                        return mix(
                            mix(
                                #3,
                                #x0,
                                self.active
                            ),
                            #x0,
                            self.hover
                        )
                    }
                }

                draw_bg: {
                    instance color: #f //(THEME_COLOR_FLOATING_BG)
                    instance color_active: #e9 //(THEME_COLOR_CTRL_HOVER)
                }
            }

            draw_bg: {
                instance color: #f9 //(THEME_COLOR_FLOATING_BG)
                border_size: 1.0
            }
        }
    }

    TranscriptionModelSelector = <View> {
        height: Fit
        align: {x: 0.0, y: 0.5}
        spacing: 10

        <Label> {
            text: "Transcription model:"
            draw_text: {
                color: #222
                text_style: {font_size: 11}
            }
        }

        transcription_model_selector = <SimpleDropDown> {
            margin: 5
            labels: ["whisper-1", "gpt-4o-transcribe", "gpt-4o-mini-transcribe"]
            values: [whisper_1, gpt_4o_transcribe, gpt_4o_mini_transcribe]

            draw_text: {
                color: #222
                text_style: {font_size: 11}
            }

            popup_menu = {
                draw_text: {
                    color: #222
                    text_style: {font_size: 11}
                }
            }
        }
    }

    VoiceSelector = <View> {
        height: Fit
        align: {x: 0.0, y: 0.5}
        spacing: 10

        <Label> {
            text: "Voice:"
            draw_text: {
                color: #222
                text_style: {font_size: 11}
            }
        }

        voice_selector = <SimpleDropDown> {
            margin: 5
            labels: ["marin", "cedar", "alloy", "shimmer", "ash", "ballad", "coral", "echo", "sage", "verse"]
            values: [marin, cedar, alloy, shimmer, ash, ballad, coral, echo, sage, verse]

            draw_text: {
                color: #222
                text_style: {font_size: 11}
            }

            popup_menu = {
                draw_text: {
                    color: #222
                    text_style: {font_size: 11}
                }
            }
        }
    }

    IconButton = <Button> {
        width: Fit, height: Fit
        draw_text: {
            text_style: <THEME_FONT_ICONS> {
                font_size: 14.
            }
            color: #5,
            color_hover: #2,
            color_focus: #2
            color_down: #5
        }
        draw_bg: {
            color_down: #0000
            border_radius: 7.
            border_size: 0.
        }
    }

    DeviceSelector = <View> {
        height: Fit
        align: {x: 0.0, y: 0.5}
        spacing: 5

        label = <Label> {
            draw_text: {
                color: #222
                text_style: {font_size: 11}
            }
        }

        device_selector = <SimpleDropDown> {
            margin: 5
            labels: ["default"]
            values: [default]

            draw_text: {
                color: #222
                text_style: {font_size: 11}
            }

            popup_menu = {
                draw_text: {
                    color: #222
                    text_style: {font_size: 11}
                }
            }
        }
    }

    MuteControl = <View> {
        width: Fit, height: Fit
        align: {x: 0.5, y: 0.5}
        cursor: Hand
        mute_button = <IconButton> {
            text: ""
        }
        mute_status = <Label> {
            padding: 0
            text: "Mute"
            draw_text: {
                color: #222
                text_style: {font_size: 11}
            }
        }
    }

    DevicesSelector = <View> {
        height: Fit, width: Fill
        flow: Down, spacing: 5
        <View> {
            height: Fit
            mic_selector = <DeviceSelector> {
                width: Fit
                label = { text: "Mic:"}
            }
            mute_control = <MuteControl> {}
        }
        speaker_selector = <DeviceSelector> {
            label = { text: "Speaker:"}
        }
    }

    Controls = <View> {
        width: Fill, height: Fit
        flow: Down
        spacing: 10
        align: {x: 0.0, y: 0.5}
        padding: 20

        devices_selector = <DevicesSelector> {}
        selected_devices_view = <View> {
            visible: false
            height: Fit
            align: {x: 0.0, y: 0.5}
            selected_devices = <Label> {
                draw_text: {
                    text_style: {font_size: 11}
                    color: #222
                }
            }
        }

        voice_selector_wrapper = <VoiceSelector> {}
        selected_voice_view = <View> {
            visible: false
            height: Fit
            align: {x: 0.0, y: 0.5}
            selected_voice = <Label> {
                draw_text: {
                    text_style: {font_size: 11}
                    color: #222
                }
            }
        }

        <TranscriptionModelSelector> {}

        toggle_interruptions = <Toggle> {
            text: "Allow interruptions\n(requires headphones, no AEC yet)"
            width: Fit
            height: Fit
            draw_text: {
                fn get_color(self) -> vec4 {
                    return #222;
                }
                text_style: {font_size: 10}
            }

            label_walk: {
                margin: {left: 50}
            }
            draw_bg: {
                size: 25.
            }

            padding: {left: 5, right: 5, top: 5, bottom: 5}
        }

        status_label = <Label> {
            text: "Ready to start"
            width: Fill
            draw_text: {
                color: #222
                wrap: Word
                text_style: {font_size: 11}
            }
        }

        request_permission_button = <RoundedShadowView> {
            visible: false
            cursor: Hand
            margin: {left: 10, right: 10, bottom: 0, top: 10}
            width: Fill, height: Fit
            align: {x: 0.5, y: 0.5}
            padding: {left: 20, right: 20, bottom: 10, top: 10}
            draw_bg: {
                color: #f9f9f9
                border_radius: 4.5,
                uniform shadow_color: #0002
                shadow_radius: 8.0,
                shadow_offset: vec2(0.0,-1.5)
            }
            <Label> {
                text: "Request microphone permission"
                draw_text: {
                    text_style: {font_size: 11}
                    color: #000
                }
            }
        }

        tool_permission_line = <ToolRequestLine> {
            visible: false
            margin: {left: 10, right: 10, top: 10}
        }

        start_stop_button = <RoundedShadowView> {
            cursor: Hand
            margin: {left: 10, right: 10, bottom: 0, top: 10}
            width: Fill, height: Fit
            align: {x: 0.5, y: 0.5}
            padding: {left: 20, right: 20, bottom: 10, top: 10}
            draw_bg: {
                color: #f9f9f9
                border_radius: 4.5,
                uniform shadow_color: #0002
                shadow_radius: 8.0,
                shadow_offset: vec2(0.0,-1.5)
            }
            stop_start_label = <Label> {
                text: "Start"
                draw_text: {
                    text_style: {font_size: 11}
                    color: #000
                }
            }
        }
    }

    pub Realtime = {{Realtime}} <RoundedView> {
        show_bg: true
        draw_bg: {
            color: #f9f9f9
            border_radius: 10.0
        }
        flow: Down
        spacing: 20
        width: Fill, height: Fit
        align: {x: 0.5, y: 0.0}
        padding: 10

        header = <View> {
            height: Fit
            flow: Overlay

            align: {x: 1.0, y: 0.5}
            close_button = <IconButton> {
                text: "" // fa-xmark
            }
        }

        <AIAnimation> {}
        <Controls> {}
    }

    pub RealtimeContent = <RoundedView> {
        align: {x: 0.5, y: 0.5}

        <AdaptiveView> {
            Desktop = {
                width: 450, height: Fit
                align: {x: 0.5, y: 0.5}

                <CachedWidget> {
                    realtime = <Realtime>{}
                }
            }

            Mobile = {
                width: Fill, height: Fill
                align: {x: 0.5, y: 0.5}

                <CachedWidget> {
                    realtime = <Realtime>{}
                }
            }
        }
    }
}

#[derive(Clone, Debug, DefaultNone)]
pub enum RealtimeModalAction {
    None,
    DismissModal,
}

#[derive(Clone, Debug, Default, PartialEq)]
enum MicPermissionStatus {
    #[default]
    NotDetermined,
    Requesting,
    Granted,
    Denied,
}

#[derive(Live, LiveHook, Widget)]
pub struct Realtime {
    #[deref]
    view: View,

    #[rust]
    realtime_channel: Option<RealtimeChannel>,

    #[rust]
    is_connected: bool,

    #[rust]
    conversation_active: bool,

    #[rust]
    transcript: String,

    #[rust]
    conversation_messages: Vec<(String, Message)>, // (item_id, message) for ordering

    #[rust]
    recorded_audio: Arc<Mutex<Vec<f32>>>,

    #[rust]
    playback_audio: Arc<Mutex<Vec<f32>>>,

    /// Whether we should record and send audio
    #[rust]
    should_record: Arc<Mutex<bool>>,

    /// Whether the user has muted the microphone
    #[rust]
    is_muted: Arc<Mutex<bool>>,

    #[rust]
    is_playing: Arc<Mutex<bool>>,

    #[rust]
    playback_position: Arc<Mutex<usize>>,

    #[rust]
    audio_setup_done: bool,

    #[rust]
    audio_streaming_timer: Option<Timer>,

    #[rust]
    ai_is_responding: bool,

    #[rust]
    user_is_interrupting: bool,

    #[rust]
    current_assistant_item_id: Option<String>,

    #[rust]
    selected_voice: String,

    #[rust]
    has_sent_audio: bool,

    #[rust]
    should_request_connection: bool,

    #[rust]
    connection_request_sent: bool,

    #[rust]
    bot_entity_id: Option<EntityId>,

    #[rust]
    bot_context: Option<crate::protocol::BotContext>,

    #[rust]
    pending_tool_call: Option<(String, String, String)>, // (name, call_id, arguments)

    #[rust]
    audio_devices: Vec<AudioDeviceDesc>,

    #[rust]
    mic_permission_status: MicPermissionStatus,
}

impl Widget for Realtime {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        if let Some(_value) = self
            .drop_down(id!(transcription_model_selector))
            .changed(event.actions())
        {
            if self.is_connected {
                self.update_session_config(cx);
            }
        }

        if let Some(enabled) = self
            .check_box(id!(toggle_interruptions))
            .changed(event.actions())
        {
            // // Send interruption configuration to the realtime client
            // if let Some(channel) = &self.realtime_channel {
            //     let _ = channel.command_sender.send(RealtimeCommand::SetInterruptionEnabled(enabled));
            // }

            if enabled && self.conversation_active {
                *self.should_record.lock().unwrap() = true;
            }
        }

        // Handle realtime events
        self.handle_realtime_events(cx);

        if !self.audio_setup_done
            && self.mic_permission_status == MicPermissionStatus::NotDetermined
        {
            cx.request_permission(Permission::AudioInput);
            self.mic_permission_status = MicPermissionStatus::Requesting;
        }

        if !self.audio_setup_done
            && let Event::PermissionResult(pr) = event
        {
            if pr.permission == Permission::AudioInput {
                match pr.status {
                    PermissionStatus::Granted => {
                        self.mic_permission_status = MicPermissionStatus::Granted;
                        self.setup_audio(cx);
                        self.audio_setup_done = true;
                        self.view(id!(start_stop_button)).set_visible(cx, true);
                    }
                    PermissionStatus::DeniedCanRetry => {
                        self.label(id!(status_label)).set_text(cx, "⚠️ Moly needs microphone access to have realtime conversations.\nClick on the button below to trigger another request");
                        self.view(id!(request_permission_button))
                            .set_visible(cx, true);
                        self.view(id!(start_stop_button)).set_visible(cx, false);
                        self.mic_permission_status = MicPermissionStatus::Denied;
                    }
                    _ => {
                        self.label(id!(status_label)).set_text(cx, "⚠️ Moly does not have access to your microphone.\nTo continue, allow Moly to access your microphone\nin your system settings\nand then restart the app.");
                        self.view(id!(request_permission_button))
                            .set_visible(cx, false);
                        self.view(id!(start_stop_button)).set_visible(cx, false);
                        self.mic_permission_status = MicPermissionStatus::Denied;
                    }
                }
            }
        }

        if self.audio_setup_done {
            // Try to start pending conversation if we got connected
            self.try_start_pending_conversation(cx);
        }

        // Handle audio streaming timer
        if let Some(timer) = &self.audio_streaming_timer {
            if timer.is_event(event).is_some() && self.conversation_active {
                self.send_audio_chunk_to_realtime(cx);

                // Check if we should resume recording when playback buffer is empty
                // This is the backup mechanism for when toggle is OFF (no interruptions)
                if self.playback_audio.lock().unwrap().is_empty() {
                    let interruptions_enabled =
                        self.check_box(id!(toggle_interruptions)).active(cx);

                    if !interruptions_enabled {
                        // Only auto-resume recording if interruptions are disabled
                        // (when interruptions are enabled, recording control is handled elsewhere)
                        if let Ok(mut should_record) = self.should_record.try_lock() {
                            if !*should_record && self.conversation_active && !self.ai_is_responding
                            {
                                ::log::debug!(
                                    "Auto-resuming recording - playback empty and interruptions disabled"
                                );
                                *should_record = true;
                                self.label(id!(status_label))
                                    .set_text(cx, "🎤 Listening...");
                            }
                        }
                    }
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for Realtime {
    /// Triggered at startup and whenever system audio devices change.
    ///
    /// We use it to update the list of available audio devices and select the default ones.
    fn handle_audio_devices(
        &mut self,
        cx: &mut Cx,
        devices: &AudioDevicesEvent,
        _scope: &mut Scope,
    ) {
        let mut input_names = Vec::new();
        let mut output_names = Vec::new();
        let mut default_input_name = String::new();
        let mut default_output_name = String::new();

        devices
            .descs
            .iter()
            .for_each(|desc| match desc.device_type {
                AudioDeviceType::Input => {
                    input_names.push(desc.name.clone());
                    if desc.is_default {
                        default_input_name = desc.name.clone();
                    }
                }
                AudioDeviceType::Output => {
                    output_names.push(desc.name.clone());
                    if desc.is_default {
                        default_output_name = desc.name.clone();
                    }
                }
            });

        let mic_dropdown = self.drop_down(id!(mic_selector.device_selector));
        mic_dropdown.set_labels(cx, input_names.clone());
        mic_dropdown.set_selected_by_label(&default_input_name, cx);

        let speaker_dropdown = self.drop_down(id!(speaker_selector.device_selector));
        speaker_dropdown.set_labels(cx, output_names.clone());
        speaker_dropdown.set_selected_by_label(&default_output_name, cx);

        // Automatically switch to default devices
        // e.g. when a user connects headphones we assume they want to use them right away.
        // Note: we do not want to automatically switch to default devices if the user has already selected a non-default device, unless
        // the default device is new (wasn't present in the previous list)
        let default_input = devices.default_input();
        let default_output = devices.default_output();

        // The default device is new, assume we want to use it
        if !self
            .audio_devices
            .iter()
            .any(|d| d.device_type == AudioDeviceType::Input && d.device_id == default_input[0])
        {
            cx.use_audio_inputs(&default_input);
        }

        // The default device is new, assume we want to use it
        if !self
            .audio_devices
            .iter()
            .any(|d| d.device_type == AudioDeviceType::Output && d.device_id == default_output[0])
        {
            cx.use_audio_outputs(&default_output);
        }

        self.audio_devices = devices.descs.clone();
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        if self
            .view(id!(start_stop_button))
            .finger_down(actions)
            .is_some()
        {
            if self.conversation_active {
                self.reset_all(cx);
            } else {
                self.start_conversation(cx);
            }
            self.update_ui(cx);
        }

        // Handle tool permission buttons from ToolRequestLine
        if self
            .view(id!(tool_permission_line))
            .button(id!(message_section.content_section.tool_actions.approve))
            .clicked(actions)
        {
            self.approve_tool_call(cx);
        }

        if self
            .view(id!(tool_permission_line))
            .button(id!(message_section.content_section.tool_actions.deny))
            .clicked(actions)
        {
            self.deny_tool_call(cx);
        }

        let speaker_dropdown = self.drop_down(id!(speaker_selector.device_selector));
        if let Some(_id) = speaker_dropdown.changed(actions) {
            let selected_device = self
                .audio_devices
                .iter()
                .find(|device| device.name == speaker_dropdown.selected_label());
            if let Some(device) = selected_device {
                cx.use_audio_outputs(&[device.device_id]);
            }
        }

        let microphone_dropdown = self.drop_down(id!(mic_selector.device_selector));
        if let Some(_id) = microphone_dropdown.changed(actions) {
            let selected_device = self
                .audio_devices
                .iter()
                .find(|device| device.name == microphone_dropdown.selected_label());
            if let Some(device) = selected_device {
                cx.use_audio_inputs(&[device.device_id]);
            }
        }

        // Mute
        let mute_button = self.button(id!(mute_button));
        let mute_label = self.label(id!(mute_status));
        if self.view(id!(mute_control)).finger_down(actions).is_some()
            || mute_button.clicked(actions)
        {
            let mut is_muted = self.is_muted.lock().unwrap();
            if *is_muted {
                // Mic was muted, unmute and update button to "Mute"
                *is_muted = false;
                mute_button.set_text(cx, ""); // fa-microphone
                mute_label.set_text(cx, "Mute");
            } else {
                *is_muted = true;
                mute_button.set_text(cx, ""); // fa-microphone-slash
                mute_label.set_text(cx, "Unmute");
            }
        }

        // Mic permissions
        if self
            .view(id!(request_permission_button))
            .finger_up(actions)
            .is_some()
        {
            cx.request_permission(Permission::AudioInput);
        }

        // Modal close
        if self.button(id!(close_button)).clicked(actions) {
            self.reset_state(cx);
            cx.action(RealtimeModalAction::DismissModal);
        }
    }
}

impl Realtime {
    pub fn set_realtime_channel(&mut self, channel: RealtimeChannel) {
        self.realtime_channel = Some(channel);
        self.is_connected = true;
    }

    pub fn set_bot_entity_id(&mut self, cx: &mut Cx, bot_entity_id: EntityId) {
        self.bot_entity_id = Some(bot_entity_id);

        // TODO: set the available transcription models through the realtime channel.
        // (determine the list of models in openai_realtime client)
        // If the provider is not OpenAI, replace `whisper-1` with `whisper`
        if let Some(EntityId::Bot(bot_id)) = &self.bot_entity_id {
            if !bot_id.provider().contains("api.openai.com") {
                let labels = vec![
                    "whisper".to_string(),
                    "gpt-4o-transcribe".to_string(),
                    "gpt-4o-mini-transcribe".to_string(),
                ];
                self.drop_down(id!(transcription_model_selector))
                    .set_labels(cx, labels);
            }
        }
    }

    pub fn set_bot_context(&mut self, bot_context: Option<crate::protocol::BotContext>) {
        self.bot_context = bot_context;
    }

    fn try_start_pending_conversation(&mut self, cx: &mut Cx) {
        if self.is_connected && !self.conversation_active && self.should_request_connection {
            // We can now start the conversation that was requested
            self.should_request_connection = false;
            self.connection_request_sent = false;
            self.conversation_active = true;
            self.ai_is_responding = true;
            self.user_is_interrupting = false;
            self.current_assistant_item_id = None;
            *self.should_record.lock().unwrap() = false;
            self.has_sent_audio = false;

            // Clear previous audio
            self.recorded_audio.lock().unwrap().clear();
            self.playback_audio.lock().unwrap().clear();
            *self.is_playing.lock().unwrap() = false;
            *self.playback_position.lock().unwrap() = 0;
            self.transcript.clear();

            self.update_ui(cx);
            self.start_audio_streaming(cx);
            self.create_greeting_response(cx);
        }
    }

    fn start_conversation(&mut self, cx: &mut Cx) {
        if !self.is_connected {
            // Set flag to request reconnection, Chat widget will handle this
            self.should_request_connection = true;
            self.connection_request_sent = false;
            self.label(id!(status_label))
                .set_text(cx, "Reconnecting...");
            return;
        }

        self.conversation_active = true;
        self.ai_is_responding = true;
        self.user_is_interrupting = false;
        self.current_assistant_item_id = None;
        *self.should_record.lock().unwrap() = false;
        self.has_sent_audio = false;

        // Clear previous audio
        self.recorded_audio.lock().unwrap().clear();
        self.playback_audio.lock().unwrap().clear();
        *self.is_playing.lock().unwrap() = false;
        *self.playback_position.lock().unwrap() = 0;
        self.transcript.clear();

        self.update_ui(cx);
        self.label(id!(status_label)).set_text(cx, "Loading..."); // This will be removed by the greeting message
        self.start_audio_streaming(cx);
        self.create_greeting_response(cx);
    }

    fn start_audio_streaming(&mut self, cx: &mut Cx) {
        // Start a timer to send audio chunks periodically
        if self.audio_streaming_timer.is_none() {
            let timer = cx.start_interval(0.020); // 20ms intervals
            self.audio_streaming_timer = Some(timer);
        }
    }

    fn send_audio_chunk_to_realtime(&mut self, _cx: &mut Cx) {
        // Collect audio data and send to realtime client
        if let Ok(mut recorded) = self.recorded_audio.try_lock() {
            if !recorded.is_empty() {
                let audio_data = recorded.clone();
                recorded.clear();

                // Convert to PCM16 and send
                let pcm16_data = Self::convert_f32_to_pcm16(&audio_data);
                if let Some(channel) = &self.realtime_channel {
                    let _ = channel
                        .command_sender
                        .unbounded_send(RealtimeCommand::SendAudio(pcm16_data));
                }
            }
        }
    }

    /// Common reset logic for both user-initiated reset and connection loss
    fn reset_conversation_state(
        &mut self,
        cx: &mut Cx,
        status_message: &str,
        allow_reconnect: bool,
    ) {
        self.stop_conversation(cx);

        self.is_connected = false;
        self.has_sent_audio = false;

        if !allow_reconnect {
            // Full reset - user clicked stop
            self.should_request_connection = false;
            self.connection_request_sent = false;
        }
        self.transcript.clear();
        self.label(id!(status_label)).set_text(cx, status_message);

        // Hide tool permission UI and clear pending tool call
        self.view(id!(tool_permission_line)).set_visible(cx, false);
        self.pending_tool_call = None;

        // Show voice selector again
        self.view(id!(voice_selector_wrapper)).set_visible(cx, true);
        self.view(id!(selected_voice_view)).set_visible(cx, false);

        self.update_ui(cx);
    }

    fn reset_all(&mut self, cx: &mut Cx) {
        self.reset_conversation_state(cx, "Ready to start", false);

        // Stop the session
        if let Some(channel) = &self.realtime_channel {
            let _ = channel
                .command_sender
                .unbounded_send(RealtimeCommand::StopSession);
        }
    }

    fn stop_conversation(&mut self, cx: &mut Cx) {
        self.conversation_active = false;
        self.ai_is_responding = false;
        self.user_is_interrupting = false;
        self.current_assistant_item_id = None;
        *self.should_record.lock().unwrap() = false;
        *self.is_playing.lock().unwrap() = false;

        // Stop audio streaming timer
        if let Some(timer) = &self.audio_streaming_timer {
            cx.stop_timer(*timer);
            self.audio_streaming_timer = None;
        }

        // Clear audio buffers
        if let Ok(mut playback) = self.playback_audio.try_lock() {
            playback.clear();
        }
        if let Ok(mut recorded) = self.recorded_audio.try_lock() {
            recorded.clear();
        }
    }

    fn handle_realtime_events(&mut self, cx: &mut Cx) {
        let events = if let Some(channel) = &self.realtime_channel {
            if let Ok(mut receiver_opt) = channel.event_receiver.lock() {
                if let Some(receiver) = receiver_opt.as_mut() {
                    let mut events = Vec::new();
                    while let Ok(Some(event)) = receiver.try_next() {
                        events.push(event);
                    }
                    events
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // Now process events without holding the lock
        for event in events {
            match event {
                RealtimeEvent::SessionReady => {
                    self.label(id!(connection_status))
                        .set_text(cx, "✅ Connected to OpenAI");
                    // self.update_session_config(cx);
                }
                RealtimeEvent::AudioData(audio_data) => {
                    // When we start receiving AI audio, the user is no longer interrupting
                    if self.user_is_interrupting {
                        self.user_is_interrupting = false;
                    }

                    self.ai_is_responding = true;

                    // Process audio immediately to start playback
                    self.add_audio_to_playback(audio_data);

                    // Update recording state based on interruption settings
                    if self.conversation_active {
                        let interruptions_enabled =
                            self.check_box(id!(toggle_interruptions)).active(cx);

                        if !interruptions_enabled {
                            // Interruptions disabled - mute microphone during AI speech
                            *self.should_record.lock().unwrap() = false;
                        } else {
                            // Interruptions enabled - ensure recording is active for real-time interruption
                            *self.should_record.lock().unwrap() = true;
                        }
                    }

                    self.label(id!(status_label))
                        .set_text(cx, "🔊 Playing audio...");
                }
                RealtimeEvent::AudioTranscript(text) => {
                    self.transcript.push_str(&text);
                }
                RealtimeEvent::AudioTranscriptCompleted(transcript, item_id) => {
                    // Store completed AI transcript as a bot message
                    if !transcript.trim().is_empty() {
                        let message = Message {
                            from: self.bot_entity_id.clone().unwrap_or_default(),
                            content: MessageContent {
                                text: transcript,
                                ..Default::default()
                            },
                            ..Default::default()
                        };
                        self.conversation_messages.push((item_id, message));
                    }
                }
                RealtimeEvent::UserTranscriptCompleted(transcript, item_id) => {
                    // Store completed user transcript as a user message
                    if !transcript.trim().is_empty() {
                        let message = Message {
                            from: EntityId::User,
                            content: MessageContent {
                                text: transcript,
                                ..Default::default()
                            },
                            ..Default::default()
                        };
                        self.conversation_messages.push((item_id, message));
                    }
                }
                RealtimeEvent::SpeechStarted => {
                    self.label(id!(status_label))
                        .set_text(cx, "🎤 User speech detected");

                    self.user_is_interrupting = true;

                    // CRITICAL: Clear the playback audio buffer to stop ongoing AI audio
                    // This prevents audio accumulation and feedback loops
                    if let Ok(mut playbook) = self.playback_audio.try_lock() {
                        let cleared_samples = playbook.len();
                        playbook.clear();
                        ::log::debug!(
                            "Cleared {} audio samples from playback buffer to prevent feedback",
                            cleared_samples
                        );
                    }

                    // Stop current playback and reset position
                    if let Ok(mut is_playing) = self.is_playing.try_lock() {
                        *is_playing = false;
                    }
                    if let Ok(mut position) = self.playback_position.try_lock() {
                        *position = 0;
                    }

                    // Resume recording immediately when user starts speaking
                    if self.conversation_active {
                        *self.should_record.lock().unwrap() = true;
                    }
                }
                RealtimeEvent::SpeechStopped => {
                    self.label(id!(status_label)).set_text(cx, "Processing...");

                    // Temporarily stop recording while waiting for response
                    if self.conversation_active {
                        *self.should_record.lock().unwrap() = false;
                    }
                }
                RealtimeEvent::ResponseCompleted => {
                    let status_label = self.label(id!(status_label));
                    self.user_is_interrupting = false;
                    self.ai_is_responding = false;
                    self.current_assistant_item_id = None;

                    // Resume recording after AI response is complete
                    if self.conversation_active {
                        // Check if interruptions are enabled via the toggle
                        let interruptions_enabled =
                            self.check_box(id!(toggle_interruptions)).active(cx);

                        if interruptions_enabled {
                            // Allow immediate interruption
                            *self.should_record.lock().unwrap() = true;
                            status_label.set_text(cx, "✅ Response generated - 🎤 listening again");
                        } else {
                            // Without interruptions, only resume when playback buffer is truly empty
                            if self.playback_audio.lock().unwrap().is_empty() {
                                ::log::debug!(
                                    "Setting should_record to true - response completed and playback empty"
                                );
                                *self.should_record.lock().unwrap() = true;
                                status_label
                                    .set_text(cx, "✅ Response generated - 🎤 listening again");
                            } else {
                                status_label
                                    .set_text(cx, "✅ Response generated - 🔊 playing audio");
                                ::log::debug!("Playback still active, keeping recording disabled");
                            }
                        }
                    }
                }
                RealtimeEvent::FunctionCallRequest {
                    name,
                    call_id,
                    arguments,
                } => {
                    // Check if dangerous mode is enabled to auto-approve function calls
                    let dangerous_mode_enabled = self
                        .bot_context
                        .as_ref()
                        .map(|ctx| {
                            ctx.tool_manager()
                                .map(|tm| tm.get_dangerous_mode_enabled())
                                .unwrap_or(false)
                        })
                        .unwrap_or(false);

                    if dangerous_mode_enabled {
                        // Auto-approve function calls in dangerous mode
                        use crate::mcp::mcp_manager::display_name_from_namespaced;
                        let display_name = display_name_from_namespaced(&name);
                        self.label(id!(status_label))
                            .set_text(cx, &format!("🔧 Auto-executing tool: {}", display_name));

                        // Execute the function call directly
                        self.handle_function_call(cx, name, call_id, arguments);
                    } else {
                        // Show permission request as usual
                        self.label(id!(status_label))
                            .set_text(cx, &format!("🔧 Tool permission requested: {}", name));

                        self.show_tool_permission_request(cx, name, call_id, arguments);
                    }
                }
                RealtimeEvent::Error(error) => {
                    ::log::error!("Realtime API error: {}", error);

                    if !self.is_connected || !self.conversation_active {
                        ::log::debug!(
                            "Ignoring error - already disconnected or conversation not active"
                        );
                        return;
                    }

                    // Check if this is a connection error
                    if error.contains("Connection lost")
                        || error.contains("Connection closed")
                        || error.contains("Failed to send")
                    {
                        // Connection was dropped - use common reset but allow reconnection
                        self.reset_conversation_state(
                            cx,
                            "❌ Connection lost. Please restart the conversation.",
                            true, // allow_reconnect
                        );
                    } else {
                        // Other types of errors - just display them
                        self.label(id!(status_label))
                            .set_text(cx, &format!("❌ Error: {}", error));

                        // Resume recording on non-connection errors
                        if self.conversation_active {
                            *self.should_record.lock().unwrap() = true;
                        }
                    }
                }
            }
        }
    }

    fn show_tool_permission_request(
        &mut self,
        cx: &mut Cx,
        name: String,
        call_id: String,
        arguments: String,
    ) {
        use crate::mcp::mcp_manager::display_name_from_namespaced;

        self.pending_tool_call = Some((name.clone(), call_id, arguments));

        let tool_line = self.view(id!(tool_permission_line));
        tool_line.set_visible(cx, true);

        // Configure the tool line
        let display_name = display_name_from_namespaced(&name);

        tool_line
            .avatar(id!(message_section.sender.avatar))
            .borrow_mut()
            .unwrap()
            .avatar = Some(crate::protocol::Picture::Grapheme("T".into()));
        tool_line
            .label(id!(message_section.sender.name))
            .set_text(cx, "Permission Request");

        let content = crate::protocol::MessageContent {
            text: format!("Tool '{}' is requesting permission to run", display_name),
            ..Default::default()
        };
        tool_line
            .slot(id!(message_section.content_section.content))
            .current()
            .as_standard_message_content()
            .set_content(cx, &content);

        tool_line
            .view(id!(message_section.content_section.tool_actions))
            .set_visible(cx, true);

        // Pause recording while waiting for permission
        *self.should_record.lock().unwrap() = false;

        self.view.redraw(cx);
    }

    fn handle_function_call(
        &mut self,
        _cx: &mut Cx,
        name: String,
        call_id: String,
        arguments: String,
    ) {
        let Some(context) = self.bot_context.as_ref().cloned() else {
            ::log::error!("No bot context available for function call");
            if let Some(channel) = &self.realtime_channel {
                let error_result = serde_json::json!({
                    "error": "Tool manager not available"
                })
                .to_string();
                let _ = channel.command_sender.unbounded_send(
                    crate::protocol::RealtimeCommand::SendFunctionCallResult {
                        call_id,
                        output: error_result,
                    },
                );
            }
            return;
        };

        let Some(tool_manager) = context.tool_manager() else {
            ::log::error!("No tool manager available for function call");
            if let Some(channel) = &self.realtime_channel {
                let error_result = serde_json::json!({
                    "error": "Tool manager not available"
                })
                .to_string();
                let _ = channel.command_sender.unbounded_send(
                    crate::protocol::RealtimeCommand::SendFunctionCallResult {
                        call_id,
                        output: error_result,
                    },
                );
            }
            return;
        };

        let channel = self.realtime_channel.clone();

        let future = async move {
            // Parse the arguments JSON
            let arguments_map = match crate::mcp::mcp_manager::parse_tool_arguments(&arguments) {
                Ok(args) => args,
                Err(e) => {
                    ::log::error!("Failed to parse function call arguments: {}", e);
                    if let Some(channel) = &channel {
                        let error_result = serde_json::json!({
                            "error": e
                        })
                        .to_string();
                        let _ = channel.command_sender.unbounded_send(
                            crate::protocol::RealtimeCommand::SendFunctionCallResult {
                                call_id,
                                output: error_result,
                            },
                        );
                    }
                    return;
                }
            };

            let result = tool_manager
                .execute_tool_call(&name, &call_id, arguments_map)
                .await;

            if let Some(channel) = &channel {
                let output = if result.is_error {
                    serde_json::json!({
                        "error": result.content
                    })
                    .to_string()
                } else {
                    result.content
                };

                let _ = channel.command_sender.unbounded_send(
                    crate::protocol::RealtimeCommand::SendFunctionCallResult { call_id, output },
                );
            }
        };

        crate::utils::asynchronous::spawn(future);
    }

    fn approve_tool_call(&mut self, cx: &mut Cx) {
        if let Some((name, call_id, arguments)) = self.pending_tool_call.take() {
            // Hide permission UI
            self.view(id!(tool_permission_line)).set_visible(cx, false);

            // Update status
            use crate::mcp::mcp_manager::display_name_from_namespaced;
            let display_name = display_name_from_namespaced(&name);
            self.label(id!(status_label))
                .set_text(cx, &format!("🔧 Executing tool: {}", display_name));

            // Execute the tool
            self.handle_function_call(cx, name, call_id, arguments);

            // Resume recording if conversation is active
            if self.conversation_active {
                *self.should_record.lock().unwrap() = true;
            }

            self.view.redraw(cx);
        }
    }

    fn deny_tool_call(&mut self, cx: &mut Cx) {
        if let Some((name, call_id, _arguments)) = self.pending_tool_call.take() {
            // Hide permission UI
            self.view(id!(tool_permission_line)).set_visible(cx, false);

            // Send denial response
            if let Some(channel) = &self.realtime_channel {
                let denial_result = serde_json::json!({
                    "error": "Tool execution denied by user"
                })
                .to_string();
                let _ = channel.command_sender.unbounded_send(
                    crate::protocol::RealtimeCommand::SendFunctionCallResult {
                        call_id,
                        output: denial_result,
                    },
                );
            }

            // Update status
            use crate::mcp::mcp_manager::display_name_from_namespaced;
            let display_name = display_name_from_namespaced(&name);
            self.label(id!(status_label))
                .set_text(cx, &format!("🚫 Tool '{}' denied", display_name));

            // Resume recording if conversation is active
            if self.conversation_active {
                *self.should_record.lock().unwrap() = true;
            }

            self.view.redraw(cx);
        }
    }

    fn setup_audio(&mut self, cx: &mut Cx) {
        let recorded_audio = self.recorded_audio.clone();
        let should_record = self.should_record.clone();
        let is_muted = self.is_muted.clone();

        // Audio input callback - capture for realtime streaming
        cx.audio_input(0, move |_info, input_buffer| {
            if let Ok(should_record_guard) = should_record.try_lock() {
                if let Ok(is_muted_guard) = is_muted.try_lock() {
                    if *should_record_guard && !*is_muted_guard {
                        if let Ok(mut recorded) = recorded_audio.try_lock() {
                            let channel = input_buffer.channel(0);

                            // Downsample from 48kHz to 24kHz by taking every other sample
                            // TODO: this is a simple decimation - for better quality, we should use proper filtering
                            for i in (0..channel.len()).step_by(2) {
                                recorded.push(channel[i]);
                            }
                        }
                    }
                }
            }
        });

        let playback_audio = self.playback_audio.clone();
        let playback_position = self.playback_position.clone();
        let is_playing = self.is_playing.clone();

        // Audio output callback - plays AI response audio
        cx.audio_output(0, move |_info, output_buffer| {
            // Always start with silence
            output_buffer.zero();

            if let Ok(mut playback) = playback_audio.try_lock() {
                if let Ok(mut pos) = playback_position.try_lock() {
                    if let Ok(mut playing) = is_playing.try_lock() {
                        // Check if we should continue playing
                        if *playing && !playback.is_empty() && *pos < playback.len() * 2 {
                            // Write to all output channels (mono -> stereo if needed)
                            let frame_count = output_buffer.frame_count();
                            let channel_count = output_buffer.channel_count();

                            let mut samples_to_drain = 0;

                            for frame_idx in 0..frame_count {
                                // Upsample from 24kHz to 48kHz by duplicating each sample
                                let sample_idx = *pos / 2; // Each 24kHz sample maps to 2 48kHz samples

                                if sample_idx < playback.len() {
                                    let audio_sample = playback[sample_idx];

                                    // Write the same sample to all output channels
                                    for channel_idx in 0..channel_count {
                                        let channel = output_buffer.channel_mut(channel_idx);
                                        channel[frame_idx] = audio_sample;
                                    }

                                    *pos += 1;

                                    // Track how many samples we can safely remove (every 2 pos increments = 1 sample)
                                    if *pos % 2 == 0 {
                                        samples_to_drain += 1;
                                    }
                                } else {
                                    // Reached end of audio data
                                    *playing = false;
                                    *pos = 0;
                                    // Drain remaining samples since we're done
                                    samples_to_drain = playback.len();
                                    break;
                                }
                            }

                            // Remove consumed samples from the front of the buffer
                            if samples_to_drain > 0 && samples_to_drain <= playback.len() {
                                playback.drain(..samples_to_drain);
                                // Adjust position since we removed samples from the front
                                *pos = (*pos).saturating_sub(samples_to_drain * 2);
                                // ::log::debug!("Drained {} samples, buffer size now: {}, pos: {}",
                                //         samples_to_drain, playback.len(), *pos);
                            }
                        } else {
                            // Not playing or no data - ensure we output silence
                            if *playing && playback.is_empty() {
                                *playing = false;
                                *pos = 0;
                            }
                        }
                    }
                }
            }
        });

        self.audio_setup_done = true;
    }

    fn add_audio_to_playback(&mut self, audio_bytes: Vec<u8>) {
        // Convert PCM16 bytes back to f32 samples
        let samples = Self::convert_pcm16_to_f32(&audio_bytes);

        if let Ok(mut playback) = self.playback_audio.try_lock() {
            // If we're not currently playing, start fresh playback immediately
            if let Ok(mut is_playing) = self.is_playing.try_lock() {
                if !*is_playing {
                    // Clear old audio data and start fresh playback
                    playback.clear();
                    *self.playback_position.lock().unwrap() = 0;
                    *is_playing = true;
                    ::log::debug!(
                        "Started fresh playback of AI response audio ({} samples)",
                        samples.len()
                    );
                }
            }

            playback.extend_from_slice(&samples);
        }
    }

    fn convert_f32_to_pcm16(samples: &[f32]) -> Vec<u8> {
        let mut pcm16_bytes = Vec::with_capacity(samples.len() * 2);

        for &sample in samples {
            let clamped = sample.max(-1.0).min(1.0);
            let pcm16_sample = (clamped * 32767.0) as i16;
            pcm16_bytes.extend_from_slice(&pcm16_sample.to_le_bytes());
        }

        pcm16_bytes
    }

    fn convert_pcm16_to_f32(bytes: &[u8]) -> Vec<f32> {
        let mut samples = Vec::with_capacity(bytes.len() / 2);

        for chunk in bytes.chunks_exact(2) {
            let pcm16_sample = i16::from_le_bytes([chunk[0], chunk[1]]);
            let f32_sample = pcm16_sample as f32 / 32767.0;
            samples.push(f32_sample);
        }

        samples
    }

    fn update_session_config(&mut self, cx: &mut Cx) {
        self.selected_voice = self.drop_down(id!(voice_selector)).selected_label();
        self.view(id!(voice_selector_wrapper))
            .set_visible(cx, false);
        self.view(id!(selected_voice_view)).set_visible(cx, true);
        self.label(id!(selected_voice)).set_text(
            cx,
            format!("Selected voice: {}", self.selected_voice).as_str(),
        );

        // Send updated session config
        if let Some(channel) = &self.realtime_channel {
            let _ = channel
                .command_sender
                .unbounded_send(RealtimeCommand::UpdateSessionConfig {
                    voice: self.selected_voice.clone(),
                    transcription_model: self
                        .drop_down(id!(transcription_model_selector))
                        .selected_label(),
                });
        }
    }

    fn create_greeting_response(&mut self, cx: &mut Cx) {
        self.update_session_config(cx);
        if let Some(channel) = &self.realtime_channel {
            let _ = channel
                .command_sender
                .unbounded_send(RealtimeCommand::CreateGreetingResponse);
        }
    }

    fn update_ui(&self, cx: &mut Cx) {
        if !self.conversation_active {
            self.label(id!(stop_start_label))
                .set_text(cx, "Start conversation");
        } else {
            self.label(id!(stop_start_label))
                .set_text(cx, "Stop conversation");
        }
    }

    /// Check if the realtime widget is requesting a new connection
    pub fn connection_requested(&mut self) -> bool {
        if self.should_request_connection && !self.is_connected && !self.connection_request_sent {
            self.connection_request_sent = true;
            true
        } else {
            false
        }
    }

    /// Get conversation messages and clear the collection
    pub fn take_conversation_messages(&mut self) -> Vec<Message> {
        let mut messages_with_ids = std::mem::take(&mut self.conversation_messages);

        // Sort by item_id to ensure chronological order
        messages_with_ids.sort_by(|a, b| a.0.cmp(&b.0));

        // Extract just the messages, maintaining the sorted order
        messages_with_ids
            .into_iter()
            .map(|(_, message)| message)
            .collect()
    }

    /// Add reset_state method for cleanup when modal closes
    pub fn reset_state(&mut self, cx: &mut Cx) {
        self.reset_all(cx);
    }
}

impl RealtimeRef {
    pub fn set_realtime_channel(&mut self, channel: RealtimeChannel) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_realtime_channel(channel);
        }
    }

    pub fn set_bot_entity_id(&mut self, cx: &mut Cx, bot_entity_id: EntityId) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_bot_entity_id(cx, bot_entity_id);
        }
    }

    pub fn connection_requested(&mut self) -> bool {
        if let Some(mut inner) = self.borrow_mut() {
            inner.connection_requested()
        } else {
            false
        }
    }

    pub fn take_conversation_messages(&mut self) -> Vec<Message> {
        if let Some(mut inner) = self.borrow_mut() {
            inner.take_conversation_messages()
        } else {
            Vec::new()
        }
    }

    pub fn reset_state(&mut self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.reset_state(cx);
        }
    }

    pub fn set_bot_context(&mut self, bot_context: Option<crate::protocol::BotContext>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_bot_context(bot_context);
        }
    }
}
