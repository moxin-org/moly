use crate::data::store::{ProviderSyncing, ProviderSyncingStatus, Store};
use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::widgets::*;
    use crate::landing::model_card::ModelCard;

    ANIMATION_SPEED = 1.2;

    ProgressBarContainer = <View> {
        width: Fill,
        height: 10,
        flow: Overlay,
        align: {x: 0.0, y: 0.5},

        background = <RoundedView> {
            visible: false,
            width: 500,
            height: 10,
            draw_bg: {
                color: #D9D9D9,
                border_radius: 2.0,
            }
        }

        progress_bar = <RoundedView> {
            visible: false,
            width: 0, height: 10,
            draw_bg: {
                color: #F3FFA2,
                border_radius: 2.0,
                instance dither: 0.9

                fn get_color(self) -> vec4 {
                    return mix(
                        #F3FFA2,
                        #B0CBC6,
                        self.pos.x + self.dither
                    )
                }

                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size)
                    sdf.box(
                        self.border_size,
                        self.border_size,
                        self.rect_size.x - (self.border_size * 2.0),
                        self.rect_size.y - (self.border_size * 2.0),
                        max(1.0, self.border_radius)
                    )
                    sdf.fill_keep(self.get_color())
                    if self.border_size > 0.0 {
                        sdf.stroke(self.get_border_color(), self.border_size)
                    }
                    return sdf.result;
                }
            }
        }
    }

    pub ModelSelectorLoading = {{ModelSelectorLoading}} {
        width: Fill,
        height: Fill,
        align: {x: 0, y: 1},
        flow: Down

        <View> {
            width: Fill,
            height: Fill,
        }

        progress_container = <ProgressBarContainer> {}

        animator: {
            line = {
                default: restart,
                restart = {
                    redraw: true,
                    from: {all: Forward {duration: (ANIMATION_SPEED)}}
                    apply: {progress_container = { progress_bar = { draw_bg: {dither: 0.6} }}}
                }
                run = {
                    redraw: true,
                    from: {all: Forward {duration: (ANIMATION_SPEED)}}
                    apply: {progress_container = { progress_bar = { draw_bg: {dither: 0.0} }}}
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelSelectorLoading {
    #[deref]
    view: View,

    #[animator]
    animator: Animator,

    #[rust]
    timer: Timer,

    #[rust]
    progress: f64,

    #[rust]
    was_syncing: bool,

    #[rust]
    last_syncing_progress: f64,

    #[rust]
    hide_timer: Timer,
}

impl Widget for ModelSelectorLoading {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.timer.is_event(event).is_some() {
            self.update_animation(cx);
        }

        if self.hide_timer.is_event(event).is_some() {
            self.hide_progress_components(cx);
        }

        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }

        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if let Some(store) = scope.data.get::<Store>() {
            // Check if we're syncing
            if let ProviderSyncingStatus::Syncing(syncing) = &store.provider_syncing_status {
                self.was_syncing = true;
                self.show_progress_components(cx);
                self.update_progress_bar(cx, syncing);
            } else if self.was_syncing {
                // We just finished syncing - animate to completion
                self.was_syncing = false;
                self.complete_progress_bar(cx);
            }
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl ModelSelectorLoading {
    pub fn update_animation(&mut self, cx: &mut Cx) {
        self.visible = true;

        if self.animator_in_state(cx, id!(line.restart)) {
            self.animator_play(cx, id!(line.run));
            self.timer = cx.start_timeout(1.5);
        } else {
            self.animator_play(cx, id!(line.restart));
        }
    }

    fn update_progress_bar(&mut self, cx: &mut Cx, syncing: &ProviderSyncing) {
        let progress_percentage = if syncing.total > 0 {
            syncing.current as f64 / syncing.total as f64
        } else {
            0.0
        };

        // Update our stored progress values
        self.progress = progress_percentage;
        self.last_syncing_progress = progress_percentage;

        // Calculate the width - 5.0 is the multiplier (500px / 100%)
        let progress_bar_width = progress_percentage * 5.0 * 100.0;

        self.view(id!(progress_container.progress_bar)).apply_over(
            cx,
            live! {
                width: (progress_bar_width)
            },
        );
    }

    fn complete_progress_bar(&mut self, cx: &mut Cx) {
        // Always animate to full width before disappearing
        // Set width to full 500px to ensure it completes
        self.view(id!(progress_container.progress_bar)).apply_over(
            cx,
            live! {
                width: 500.0
            },
        );

        // Set a timer to hide the progress bar components after a short delay
        self.hide_timer = cx.start_timeout(0.3);
    }

    fn hide_progress_components(&mut self, cx: &mut Cx) {
        self.view(id!(progress_container.background))
            .set_visible(cx, false);

        self.view(id!(progress_container.progress_bar)).apply_over(
            cx,
            live! {
                visible: false,
                width: 0.0
            },
        );

        self.redraw(cx);
    }

    fn show_progress_components(&mut self, cx: &mut Cx) {
        self.view(id!(progress_container.background))
            .set_visible(cx, true);
        self.view(id!(progress_container.progress_bar))
            .set_visible(cx, true);
    }
}

impl ModelSelectorLoadingRef {
    pub fn show_and_animate(&mut self, cx: &mut Cx) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.visible = true;
        if inner.timer.is_empty() {
            inner.timer = cx.start_timeout(0.2);
        }
    }
}
