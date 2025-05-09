use makepad_widgets::*;

live_design! {
    // The default theme for MolyKit. 
    // 
    // Instead of overriding Makepad's default theme (under link::theme::*),
    // we create a MolyKit-specific theme (under link::widgets::*).
    // 
    // This way, we can keep Makepad's default theme as is, to not interfer with the rest of the application 
    // (and because there's no easy way to override Makepad's default theme without copy-pasting the entire theme, which would quickly become deprecated).
    // 
    // MolyKit's theme is applied via the `cx.link(live_id!(moly_kit_theme), live_id!(<some_theme>));`
    // line in the `live_design` macro in `src/widgets.rs`.
    link theme_moly_kit_light;

    use link::theme::*;
    use link::widgets::*;
    
    // TODO: In this file we'll set a set of rules/constants that will be used to style MolyKit's widgets.
    // We can also here override Makepad's default theme as needed (e.g. THEME_SPACE_FACTOR).
    //
    // Currently we're using this to globally (MolyKit-wide) override some painful defaults in Makepad.
    // Ideally we'd override some spacing values in Makepad, but that doesn't seem to be enough, 
    // therefore we're also overriding some widget-specific values here.
    pub Label = <Label> { padding: 0 }
}
