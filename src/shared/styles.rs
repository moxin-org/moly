use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use makepad_draw::shader::std::*;

    pub MODEL_LINK_FONT_COLOR = #x155EEF
    pub SIDEBAR_BG_COLOR = #f2f2f2
    pub MAIN_BG_COLOR = #f9f9f9
    pub MAIN_BG_COLOR_DARK = #f2f2f2
    pub CTA_BUTTON_COLOR = #3A7CA5

    pub REGULAR_FONT = <THEME_FONT_REGULAR>{
        font_size: (12)
    }

    pub BOLD_FONT = <THEME_FONT_BOLD>{
        font_size: (12)
    }

    pub RoundedInnerShadowView = <View> {
        show_bg: true,
        draw_bg: {
            color: #8
            uniform border_radius: 2.5
            uniform border_size: 0.0
            uniform border_color: #0000
            uniform shadow_color: #0007
            uniform shadow_radius: 10.0,
            uniform shadow_offset: vec2(0.0,0.0)
                        
            fn get_color(self) -> vec4 {
                return self.color
            }
                        
            fn get_border_color(self) -> vec4 {
                return self.border_color
            }
                            
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                
                // Main shape definition
                let outer_x = self.border_size;
                let outer_y = self.border_size;
                let outer_w = self.rect_size.x - 2.0 * self.border_size;
                let outer_h = self.rect_size.y - 2.0 * self.border_size;
                let outer_rad = max(1.0, self.border_radius);
                
                sdf.box(outer_x, outer_y, outer_w, outer_h, outer_rad);
                let outer_dist = sdf.shape; // Distance from the outer edge. Negative inside.

                // Base color (non-premultiplied)
                let base_color_raw = self.get_color();
                 
                // Calculate shadow parameters
                let shadow_blur = self.shadow_radius;
                // TODO: Incorporate shadow_offset correctly
                
                // Calculate the distance from the edge, inside the shape.
                // outer_dist is negative inside. -outer_dist is positive inside, 0 at the edge.
                let dist_from_edge_inside = -outer_dist; 
                
                // Map distance [0, shadow_blur] to intensity [1, 0] using smoothstep for a nice fade.
                let intensity = 1.0 - smoothstep(0.0, shadow_blur, dist_from_edge_inside);
                
                // Clamp intensity and ensure it's only applied inside the shape (outer_dist <= 0)
                // shadow_factor is 1 near edge inside, fading to 0 further inside. Only > 0 inside.
                let shadow_factor = clamp(intensity, 0.0, 1.0) * step(outer_dist, 0.0); 

                // Calculate effective alpha for blending towards shadow color RGB
                let effective_shadow_alpha = shadow_factor * self.shadow_color.a;

                // Blend base color RGB towards shadow color RGB based on the shadow factor
                let final_rgb = mix(base_color_raw.rgb, self.shadow_color.rgb, effective_shadow_alpha);
                
                // Final color uses the original base alpha
                let final_color_raw = vec4(final_rgb, base_color_raw.a);

                // Fill the shape with this calculated final color. 
                // sdf.fill handles premultiplication and uses sdf.shape for masking/AA.
                sdf.fill(final_color_raw); 
                
                // Apply border if needed (drawn on top of the fill)
                if self.border_size > 0.0 {
                   // Need to redefine shape before stroke, as fill resets sdf.shape
                   sdf.box(outer_x, outer_y, outer_w, outer_h, outer_rad); 
                   // stroke_keep expects a non-premultiplied color
                   sdf.stroke_keep(self.get_border_color(), self.border_size); 
                }
                
                // sdf.result now contains the premultiplied final color (fill + optional border)
                return sdf.result;
            }
        }
    }
}
