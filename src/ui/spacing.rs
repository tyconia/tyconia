//! Consistent spacing tokens for the UI stack.
//!
//! Tokens are expressed directly in logical pixels. `UI_SCALE` remains the base
//! unit for elements that intentionally scale with the UI, use [`ui_val`] /
//! [`ui_units`] for those.

use bevy::prelude::*;

use crate::ui::UI_SCALE;

pub const SPACING_XS: f32 = 4.0;
pub const SPACING_SM: f32 = 8.0;
pub const SPACING_MD: f32 = 16.0;
pub const SPACING_LG: f32 = 24.0;
pub const SPACING_XL: f32 = 32.0;

/// Convert a multiple of [`UI_SCALE`] into logical pixels.
pub fn ui_units(multiplier: f32) -> f32 {
    multiplier * UI_SCALE
}

/// Convert a multiple of [`UI_SCALE`] into a [`Val::Px`].
pub fn ui_val(multiplier: f32) -> Val {
    Val::Px(ui_units(multiplier))
}

pub fn spacing_xs() -> Val {
    Val::Px(SPACING_XS)
}

pub fn spacing_sm() -> Val {
    Val::Px(SPACING_SM)
}

pub fn spacing_md() -> Val {
    Val::Px(SPACING_MD)
}

pub fn spacing_lg() -> Val {
    Val::Px(SPACING_LG)
}

pub fn spacing_xl() -> Val {
    Val::Px(SPACING_XL)
}

pub fn rect_all(px: f32) -> UiRect {
    UiRect::all(Val::Px(px))
}

pub fn rect_axes(horizontal: f32, vertical: f32) -> UiRect {
    UiRect::axes(Val::Px(horizontal), Val::Px(vertical))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Spacing {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    /// Custom value in logical pixels.
    Custom(f32),
}

impl Spacing {
    pub fn val(self) -> Val {
        Val::Px(self.px())
    }

    pub fn px(self) -> f32 {
        match self {
            Spacing::Xs => SPACING_XS,
            Spacing::Sm => SPACING_SM,
            Spacing::Md => SPACING_MD,
            Spacing::Lg => SPACING_LG,
            Spacing::Xl => SPACING_XL,
            Spacing::Custom(v) => v,
        }
    }

    pub fn rect_all(self) -> UiRect {
        UiRect::all(self.val())
    }

    pub fn rect_axes(self, vertical: Spacing) -> UiRect {
        UiRect::axes(self.val(), vertical.val())
    }
}

impl From<Spacing> for Val {
    fn from(s: Spacing) -> Self {
        s.val()
    }
}

impl From<Spacing> for UiRect {
    fn from(s: Spacing) -> Self {
        s.rect_all()
    }
}

/// Multiples of `UI_SCALE` used for element sizing.
pub const WINDOW_TITLE_BAR_HEIGHT: f32 = 6.0;
pub const WINDOW_MIN_WIDTH: f32 = 200.0;
pub const WINDOW_MIN_HEIGHT: f32 = 150.0;

pub const INVENTORY_SLOT_SIZE: f32 = 8.0;
pub const INVENTORY_SLOT_ASPECT: f32 = 17.0 / 18.0;

pub const DROPDOWN_MAX_HEIGHT: f32 = 300.0;

pub const TOOLTIP_MAX_WIDTH: f32 = 300.0;
pub const TOOLTIP_PADDING: Spacing = Spacing::Custom(12.0);
pub const TOOLTIP_DELAY: f32 = 0.4;