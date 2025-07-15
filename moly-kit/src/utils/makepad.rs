//! Utilities to deal with stuff that is highly specific to Makepad.

mod events;
mod portal_list;

pub use events::*;
pub(crate) use portal_list::*;

use makepad_widgets::*;

/// Convert from hex color notation to makepad's Vec4 color.
/// Ex: Converts `0xff33cc` into `vec4(1.0, 0.2, 0.8, 1.0)`.
pub fn hex_rgb_color(hex: u32) -> Vec4 {
    let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
    let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
    let b = (hex & 0xFF) as f32 / 255.0;
    vec4(r, g, b, 1.0)
}
