use crate::{protocol::*, utils::makepad::events::EventExt};
use makepad_widgets::*;
use std::sync::{Arc, Mutex};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

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

        transcription_model_selector = <DropDown> {
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

        voice_selector = <DropDown> {
            margin: 5
            labels: ["alloy", "shimmer", "ash", "ballad", "coral", "echo", "sage", "verse"]
            values: [alloy, shimmer, ash, ballad, coral, echo, sage, verse]

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

    pub Realtime = {{Realtime}} <RoundedView> {
        show_bg: true
        draw_bg: {
            color: #f9f9f9
            border_radius: 10.0
        }
        flow: Down
        width: 450, height: 600
        align: {x: 0.5, y: 0.5}

        <RoundedView> {
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

        controls = <View> {
            width: Fill, height: Fit
            flow: Down
            spacing: 10
            align: {x: 0.0, y: 0.5}
            padding: 20

            <TranscriptionModelSelector> {}
            voice_selector_wrapper = <VoiceSelector> {}
            selected_voice_view = <View> {
                visible: false
                height: Fit
                align: {x: 0.5, y: 0.5}
                selected_voice = <Label> {
                    draw_text: {
                        text_style: {font_size: 11}
                        color: #222
                    }
                }
            }

            // <View> {
            //     height: Fit
            //     align: {x: 0.5, y: 0.5}
            //     spacing: 20
            //     flow: Right

            //     connect_button = <Button> {
            //         text: "🔗 Connect and start conversation"
            //         draw_text: {text_style: {font_size: 11}}
            //     }

            //     connection_status = <Label> {
            //         text: "Disconnected"
            //         draw_text: {text_style: {font_size: 11}}
            //     }
            // }

            toggle_interruptions = <Toggle> {
                text: "Allow interruptions\n(requires headphones, no AEC yet)"
                width: Fit
                height: Fit
                draw_text: {
                    color: #222
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

            // transcript_area = <RoundedView> {
            //     width: Fill, height: 50
            //     show_bg: true
            //     draw_bg: {
            //         color: #f0f0f0
            //         border_radius: 5.0
            //     }
            //     padding: 10

            //     transcript_label = <Label> {
            //         text: "Transcript will appear here..."
            //         width: Fill
            //         draw_text: {
            //             color: #222
            //             text_style: {font_size: 10.5}
            //         }
            //         align: {x: 0.0, y: 0.0}
            //     }
            // }

            status_label = <Label> {
                text: "Ready to connect"
                width: Fill
                draw_text: {
                    color: #222
                    text_style: {font_size: 11}
                }
                align: {x: 0.5, y: 0.5}
            }

            reset_button = <RoundedShadowView> {
                cursor: Hand
                margin: {left: 10, right: 10, bottom: 0}
                width: Fill, height: Fit
                align: {x: 0.5, y: 0.5}
                padding: {left: 30, right: 30, bottom: 15, top: 15}
                draw_bg: {
                    color: #f9f9f9
                    border_radius: 4.5,
                    uniform shadow_color: #0002
                    shadow_radius: 8.0,
                    shadow_offset: vec2(0.0,-1.5)
                }
                <Label> {
                    text: "🔄 Reset"
                    draw_text: {
                        text_style: {font_size: 11}
                        color: #000
                    }
                }
            }
        }
    }
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
    recorded_audio: Arc<Mutex<Vec<f32>>>,

    #[rust]
    playback_audio: Arc<Mutex<Vec<f32>>>,

    #[rust]
    is_recording: Arc<Mutex<bool>>,

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
}

impl Widget for Realtime {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        // Handle button clicks
        if self.button(id!(connect_button)).clicked(event.actions()) {
            self.connect_and_start_conversation(cx);
        }

        if self
            .view(id!(reset_button))
            .finger_up(event.actions())
            .is_some()
        {
            self.reset_all(cx);
        }

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
                *self.is_recording.lock().unwrap() = true;
            }
        }

        // Handle realtime events
        self.handle_realtime_events(cx);

        // Setup audio if needed
        if !self.audio_setup_done {
            self.setup_audio(cx);
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
                        if let Ok(mut is_recording) = self.is_recording.try_lock() {
                            if !*is_recording && self.conversation_active && !self.ai_is_responding
                            {
                                log!(
                                    "Auto-resuming recording - playback empty and interruptions disabled"
                                );
                                *is_recording = true;
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
    fn handle_audio_devices(
        &mut self,
        cx: &mut Cx,
        devices: &AudioDevicesEvent,
        _scope: &mut Scope,
    ) {
        log!(
            "App::handle_audio_devices called with {} devices",
            devices.descs.len()
        );
        for _desc in &devices.descs {
            // log!("Audio device: {}", desc);
        }

        // Use default input and output devices
        let default_input = devices.default_input();
        let default_output = devices.default_output();

        log!("Default input: {:?}", default_input);
        log!("Default output: {:?}", default_output);

        cx.use_audio_inputs(&default_input);
        cx.use_audio_outputs(&default_output);
    }
}

impl Realtime {
    pub fn set_realtime_channel(&mut self, channel: RealtimeChannel) {
        self.realtime_channel = Some(channel);
        self.is_connected = true;
        log!("Realtime channel set");
    }

    fn connect_and_start_conversation(&mut self, cx: &mut Cx) {
        if let Some(channel) = &self.realtime_channel {
            self.is_connected = true;
            self.label(id!(connection_status))
                .set_text(cx, "🔄 Connecting...");

            // Start the session and trigger greeting
            let _ = channel.command_sender.send(RealtimeCommand::StartSession);
            log!("Starting realtime session");
        } else {
            self.label(id!(status_label))
                .set_text(cx, "No realtime channel available");
        }
    }

    fn start_conversation(&mut self, cx: &mut Cx) {
        if !self.is_connected {
            self.label(id!(status_label))
                .set_text(cx, "❌ Not connected");
            return;
        }

        log!("Starting conversation");
        self.conversation_active = true;
        self.ai_is_responding = false;
        self.user_is_interrupting = false;
        self.current_assistant_item_id = None;
        *self.is_recording.lock().unwrap() = true;
        self.has_sent_audio = false;

        // Clear previous audio
        self.recorded_audio.lock().unwrap().clear();
        self.playback_audio.lock().unwrap().clear();
        *self.is_playing.lock().unwrap() = false;
        *self.playback_position.lock().unwrap() = 0;
        self.transcript.clear();

        // Update UI state
        self.update_ui_state(cx);

        // Start audio streaming
        self.start_audio_streaming(cx);

        // Create AI greeting response
        self.create_greeting_response();

        log!("Conversation started with AI greeting");
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

                // log!("Sending audio chunk to realtime");
                // Convert to PCM16 and send
                let pcm16_data = Self::convert_f32_to_pcm16(&audio_data);
                if let Some(channel) = &self.realtime_channel {
                    let _ = channel
                        .command_sender
                        .send(RealtimeCommand::SendAudio(pcm16_data));
                }
            } else {
                // log!("No audio data to send");
            }
        }
    }

    fn reset_all(&mut self, cx: &mut Cx) {
        self.stop_conversation(cx);

        self.is_connected = false;
        self.has_sent_audio = false;
        self.transcript.clear();
        self.label(id!(status_label))
            .set_text(cx, "Ready to connect");
        self.label(id!(transcript_label))
            .set_text(cx, "Transcript will appear here...");
        self.label(id!(connection_status))
            .set_text(cx, "Disconnected");

        // Show voice selector again
        self.view(id!(voice_selector_wrapper)).set_visible(cx, true);
        self.view(id!(selected_voice_view)).set_visible(cx, false);

        self.update_ui_state(cx);

        // Stop the session
        if let Some(channel) = &self.realtime_channel {
            let _ = channel.command_sender.send(RealtimeCommand::StopSession);
        }
    }

    fn stop_conversation(&mut self, cx: &mut Cx) {
        log!("Stopping conversation");
        self.conversation_active = false;
        self.ai_is_responding = false;
        self.user_is_interrupting = false;
        self.current_assistant_item_id = None;
        *self.is_recording.lock().unwrap() = false;
        *self.is_playing.lock().unwrap() = false;

        // Stop audio streaming timer
        if let Some(timer) = &self.audio_streaming_timer {
            cx.stop_timer(*timer);
            self.audio_streaming_timer = None;
        }

        // Cancel any pending audio playback
        if let Ok(mut playback) = self.playback_audio.try_lock() {
            playback.clear();
        }

        self.label(id!(status_label))
            .set_text(cx, "⏹️ Conversation stopped");
    }

    fn handle_realtime_events(&mut self, cx: &mut Cx) {
        let events = if let Some(channel) = &self.realtime_channel {
            if let Ok(mut receiver_opt) = channel.event_receiver.lock() {
                if let Some(receiver) = receiver_opt.as_mut() {
                    let mut events = Vec::new();
                    while let Ok(event) = receiver.try_recv() {
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
                    log!("Session ready");
                    self.label(id!(connection_status))
                        .set_text(cx, "✅ Connected to OpenAI");
                    self.update_session_config(cx);
                }
                RealtimeEvent::SessionConfigured => {
                    log!("Session configured");
                    self.label(id!(status_label))
                        .set_text(cx, "✅ Session configured");
                    self.start_conversation(cx);
                }
                RealtimeEvent::AudioData(audio_data) => {
                    // When we start receiving AI audio, the user is no longer interrupting
                    if self.user_is_interrupting {
                        log!("AI audio received - user interruption ended");
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
                            *self.is_recording.lock().unwrap() = false;
                        } else {
                            // Interruptions enabled - ensure recording is active for real-time interruption
                            *self.is_recording.lock().unwrap() = true;
                        }
                    }

                    self.label(id!(status_label))
                        .set_text(cx, "🔊 Playing audio...");
                }
                RealtimeEvent::AudioTranscript(text) => {
                    self.transcript.push_str(&text);
                    self.label(id!(transcript_label))
                        .set_text(cx, &self.transcript);
                }
                RealtimeEvent::SpeechStarted => {
                    log!("Speech detected - interrupting AI audio");
                    self.label(id!(status_label))
                        .set_text(cx, "🎤 User speech detected");

                    self.user_is_interrupting = true;

                    // CRITICAL: Clear the playback audio buffer to stop ongoing AI audio
                    // This prevents audio accumulation and feedback loops
                    if let Ok(mut playbook) = self.playback_audio.try_lock() {
                        let cleared_samples = playbook.len();
                        playbook.clear();
                        log!(
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
                        *self.is_recording.lock().unwrap() = true;
                    }
                }
                RealtimeEvent::SpeechStopped => {
                    log!("Speech ended, processing...");
                    self.label(id!(status_label))
                        .set_text(cx, "🤔 Processing...");

                    // Temporarily stop recording while waiting for response
                    if self.conversation_active {
                        *self.is_recording.lock().unwrap() = false;
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
                            *self.is_recording.lock().unwrap() = true;
                            status_label.set_text(cx, "✅ Response generated - 🎤 listening again");
                        } else {
                            // Without interruptions, only resume when playback buffer is truly empty
                            if self.playback_audio.lock().unwrap().is_empty() {
                                log!(
                                    "Setting is_recording to true - response completed and playback empty"
                                );
                                *self.is_recording.lock().unwrap() = true;
                                status_label
                                    .set_text(cx, "✅ Response generated - 🎤 listening again");
                            } else {
                                status_label
                                    .set_text(cx, "✅ Response generated - 🔊 playing audio");
                                log!("Playback still active, keeping recording disabled");
                            }
                        }
                    }
                }
                RealtimeEvent::Error(error) => {
                    log!("Realtime API error: {}", error);
                    self.label(id!(status_label))
                        .set_text(cx, &format!("❌ Error: {}", error));

                    // Resume recording on error
                    if self.conversation_active {
                        *self.is_recording.lock().unwrap() = true;
                    }
                }
            }
        }
    }

    fn setup_audio(&mut self, cx: &mut Cx) {
        log!("Setting up audio");
        let recorded_audio = self.recorded_audio.clone();
        let is_recording = self.is_recording.clone();

        // Audio input callback - capture for realtime streaming
        cx.audio_input(0, move |_info, input_buffer| {
            if let Ok(is_recording_guard) = is_recording.try_lock() {
                if *is_recording_guard {
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
                                // log!("Drained {} samples, buffer size now: {}, pos: {}",
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
                    log!(
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
                .send(RealtimeCommand::UpdateSessionConfig {
                    voice: self.selected_voice.clone(),
                    transcription_model: self
                        .drop_down(id!(transcription_model_selector))
                        .selected_label(),
                });
        }
    }

    fn create_greeting_response(&mut self) {
        if let Some(channel) = &self.realtime_channel {
            let _ = channel
                .command_sender
                .send(RealtimeCommand::CreateGreetingResponse);
        }
    }

    fn update_ui_state(&self, cx: &mut Cx) {
        // Update button states based on connection and conversation status
        if !self.is_connected {
            self.button(id!(connect_button))
                .set_text(cx, "🔗 Connect and start conversation");
        } else if self.conversation_active {
            self.button(id!(connect_button))
                .set_text(cx, "✅ Connected");
        } else {
            self.button(id!(connect_button))
                .set_text(cx, "✅ Connected");
        }
    }
}

impl RealtimeRef {
    pub fn set_realtime_channel(&mut self, channel: RealtimeChannel) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_realtime_channel(channel);
        }
    }
}
