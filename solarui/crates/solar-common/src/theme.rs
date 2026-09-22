use material_colors::color::Argb;
use material_colors::theme::ThemeBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct M3Theme {
    pub primary: String,
    pub on_primary: String,
    pub primary_container: String,
    pub on_primary_container: String,

    pub secondary: String,
    pub on_secondary: String,
    pub secondary_container: String,
    pub on_secondary_container: String,

    pub tertiary: String,
    pub on_tertiary: String,
    pub tertiary_container: String,
    pub on_tertiary_container: String,

    pub surface: String,
    pub on_surface: String,
    pub surface_variant: String,
    pub on_surface_variant: String,
    pub surface_container_lowest: String,
    pub surface_container_low: String,
    pub surface_container: String,
    pub surface_container_high: String,
    pub surface_container_highest: String,

    pub outline: String,
    pub outline_variant: String,

    pub error: String,
    pub on_error: String,

    // Motion Easing Curves
    pub easing_emphasized: String,
    pub easing_emphasized_decelerate: String,
    pub easing_emphasized_accelerate: String,
    pub easing_standard: String,

    // Motion Durations (ms)
    pub duration_short: u32,
    pub duration_medium: u32,
    pub duration_long: u32,

    // Dimensions / Radius
    pub corner_small: u32,
    pub corner_medium: u32,
    pub corner_large: u32,
    pub corner_full: u32,
}

impl Default for M3Theme {
    fn default() -> Self {
        // BlazeOS Solar Dark (Deep obsidian with Blaze electric cyan/blue accents)
        Self::from_seed_color(0xFF80D4FF)
    }
}

impl M3Theme {
    pub fn from_seed_color(seed_argb_u32: u32) -> Self {
        let theme = ThemeBuilder::with_source(Argb::from_u32(seed_argb_u32)).build();
        let d = theme.schemes.dark;

        Self {
            primary: argb_to_hex(d.primary),
            on_primary: argb_to_hex(d.on_primary),
            primary_container: argb_to_hex(d.primary_container),
            on_primary_container: argb_to_hex(d.on_primary_container),

            secondary: argb_to_hex(d.secondary),
            on_secondary: argb_to_hex(d.on_secondary),
            secondary_container: argb_to_hex(d.secondary_container),
            on_secondary_container: argb_to_hex(d.on_secondary_container),

            tertiary: argb_to_hex(d.tertiary),
            on_tertiary: argb_to_hex(d.on_tertiary),
            tertiary_container: argb_to_hex(d.tertiary_container),
            on_tertiary_container: argb_to_hex(d.on_tertiary_container),

            surface: argb_to_hex(d.surface),
            on_surface: argb_to_hex(d.on_surface),
            surface_variant: argb_to_hex(d.surface_variant),
            on_surface_variant: argb_to_hex(d.on_surface_variant),
            surface_container_lowest: "#0b0e10".to_string(),
            surface_container_low: "#15181a".to_string(),
            surface_container: "#1b1e20".to_string(),
            surface_container_high: "#25282b".to_string(),
            surface_container_highest: "#303437".to_string(),

            outline: argb_to_hex(d.outline),
            outline_variant: argb_to_hex(d.outline_variant),

            error: argb_to_hex(d.error),
            on_error: argb_to_hex(d.on_error),

            // Google Material 3 Standard Motion
            easing_emphasized: "cubic-bezier(0.2, 0.0, 0.0, 1.0)".to_string(),
            easing_emphasized_decelerate: "cubic-bezier(0.05, 0.7, 0.1, 1.0)".to_string(),
            easing_emphasized_accelerate: "cubic-bezier(0.3, 0.0, 0.8, 0.15)".to_string(),
            easing_standard: "cubic-bezier(0.4, 0.0, 0.2, 1.0)".to_string(),

            duration_short: 150,
            duration_medium: 300,
            duration_long: 500,

            corner_small: 8,
            corner_medium: 16,
            corner_large: 24,
            corner_full: 9999,
        }
    }

    pub fn to_gtk_css(&self) -> String {
        format!(
            r#"
            /* Google Material 3 Design Tokens (BlazeOS Edition) */
            @define-color md_sys_color_primary {};
            @define-color md_sys_color_on_primary {};
            @define-color md_sys_color_primary_container {};
            @define-color md_sys_color_on_primary_container {};

            @define-color md_sys_color_secondary {};
            @define-color md_sys_color_on_secondary {};
            @define-color md_sys_color_secondary_container {};
            @define-color md_sys_color_on_secondary_container {};

            @define-color md_sys_color_tertiary {};
            @define-color md_sys_color_on_tertiary {};

            @define-color md_sys_color_surface {};
            @define-color md_sys_color_on_surface {};
            @define-color md_sys_color_surface_variant {};
            @define-color md_sys_color_on_surface_variant {};

            @define-color md_sys_color_surface_container_lowest {};
            @define-color md_sys_color_surface_container_low {};
            @define-color md_sys_color_surface_container {};
            @define-color md_sys_color_surface_container_high {};
            @define-color md_sys_color_surface_container_highest {};

            @define-color md_sys_color_outline {};
            @define-color md_sys_color_outline_variant {};
            @define-color md_sys_color_error {};

            * {{
                font-family: 'Google Sans', 'Roboto Flex', 'Inter', sans-serif;
            }}

            .solar-bar {{
                background-color: alpha(@md_sys_color_surface_container_low, 0.90);
                border-radius: {}px;
                border: 1px solid alpha(@md_sys_color_outline_variant, 0.35);
                box-shadow: 0 4px 12px rgba(0, 0, 0, 0.35);
                padding: 4px 12px;
            }}

            .solar-pill {{
                background-color: @md_sys_color_surface_container_high;
                color: @md_sys_color_on_surface;
                border-radius: {}px;
                padding: 6px 14px;
                margin: 2px 4px;
                transition: all {}ms {};
            }}

            .solar-pill:hover {{
                background-color: @md_sys_color_primary_container;
                color: @md_sys_color_on_primary_container;
            }}

            .solar-pill-active {{
                background-color: @md_sys_color_primary;
                color: @md_sys_color_on_primary;
                font-weight: 600;
            }}

            .solar-quick-settings {{
                background-color: alpha(@md_sys_color_surface_container, 0.96);
                border-radius: {}px;
                border: 1px solid alpha(@md_sys_color_outline_variant, 0.4);
                padding: 20px;
                box-shadow: 0 12px 32px rgba(0, 0, 0, 0.5);
            }}
            "#,
            self.primary,
            self.on_primary,
            self.primary_container,
            self.on_primary_container,
            self.secondary,
            self.on_secondary,
            self.secondary_container,
            self.on_secondary_container,
            self.tertiary,
            self.on_tertiary,
            self.surface,
            self.on_surface,
            self.surface_variant,
            self.on_surface_variant,
            self.surface_container_lowest,
            self.surface_container_low,
            self.surface_container,
            self.surface_container_high,
            self.surface_container_highest,
            self.outline,
            self.outline_variant,
            self.error,
            self.corner_full,
            self.corner_full,
            self.duration_short,
            self.easing_emphasized,
            self.corner_large
        )
    }
}

fn argb_to_hex(argb: Argb) -> String {
    format!("#{:02x}{:02x}{:02x}", argb.red, argb.green, argb.blue)
}
