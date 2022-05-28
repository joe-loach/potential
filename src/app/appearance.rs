use archie_egui::egui;

pub(super) fn style() -> egui::Style {
    use egui::epaint::Shadow;
    use egui::{
        style::{Interaction, Margin, Selection, Spacing, Visuals, WidgetVisuals, Widgets},
        vec2, Color32, FontFamily, FontId, Rounding, Stroke, Style, TextStyle,
    };
    use std::collections::BTreeMap;

    Style {
        override_font_id: None,
        override_text_style: None,
        text_styles: {
            const SMALL: f32 = 10.0;
            const MEDIUM: f32 = 14.0;
            const LARGE: f32 = 18.0;
            let mut text_styles = BTreeMap::new();
            text_styles.insert(
                TextStyle::Small,
                FontId::new(SMALL, FontFamily::Proportional),
            );
            text_styles.insert(
                TextStyle::Body,
                FontId::new(MEDIUM, FontFamily::Proportional),
            );
            text_styles.insert(
                TextStyle::Button,
                FontId::new(MEDIUM, FontFamily::Proportional),
            );
            text_styles.insert(
                TextStyle::Heading,
                FontId::new(LARGE, FontFamily::Proportional),
            );
            text_styles.insert(
                TextStyle::Monospace,
                FontId::new(MEDIUM, FontFamily::Monospace),
            );
            text_styles
        },
        wrap: None,
        spacing: Spacing {
            item_spacing: vec2(4.0, 4.0),
            window_margin: Margin::same(4.0),
            button_padding: vec2(4.0, 1.0),
            indent: 18.0, // match checkbox/radio-button with `button_padding.x + icon_width + icon_spacing`
            interact_size: vec2(40.0, 18.0),
            slider_width: 100.0,
            text_edit_width: 280.0,
            icon_width: 14.0,
            icon_spacing: 0.0,
            tooltip_width: 600.0,
            combo_height: 200.0,
            scroll_bar_width: 8.0,
            indent_ends_with_horizontal_line: false,
        },
        interaction: Interaction::default(),
        visuals: Visuals {
            dark_mode: true,
            override_text_color: None,
            widgets: Widgets {
                noninteractive: WidgetVisuals {
                    bg_fill: Color32::from_gray(27), // window background
                    bg_stroke: Stroke::new(1.0, Color32::from_gray(60)), // separators, indentation lines, windows outlines
                    fg_stroke: Stroke::new(1.0, Color32::from_gray(140)), // normal text color
                    rounding: Rounding::same(0.0),
                    expansion: 0.0,
                },
                inactive: WidgetVisuals {
                    bg_fill: Color32::from_gray(60), // button background
                    bg_stroke: Default::default(),
                    fg_stroke: Stroke::new(1.0, Color32::from_gray(180)), // button text
                    rounding: Rounding::same(0.0),
                    expansion: 0.0,
                },
                hovered: WidgetVisuals {
                    bg_fill: Color32::from_gray(70),
                    bg_stroke: Stroke::new(1.0, Color32::from_gray(150)), // e.g. hover over window edge or button
                    fg_stroke: Stroke::new(1.5, Color32::from_gray(240)),
                    rounding: Rounding::same(0.0),
                    expansion: 1.0,
                },
                active: WidgetVisuals {
                    bg_fill: Color32::from_gray(55),
                    bg_stroke: Stroke::new(1.0, Color32::WHITE),
                    fg_stroke: Stroke::new(2.0, Color32::WHITE),
                    rounding: Rounding::same(0.0),
                    expansion: 1.0,
                },
                open: WidgetVisuals {
                    bg_fill: Color32::from_gray(27),
                    bg_stroke: Stroke::new(1.0, Color32::from_gray(60)),
                    fg_stroke: Stroke::new(1.0, Color32::from_gray(210)),
                    rounding: Rounding::same(0.0),
                    expansion: 0.0,
                },
            },
            selection: Selection::default(),
            hyperlink_color: Color32::from_rgb(90, 170, 255),
            faint_bg_color: Color32::from_gray(24),
            extreme_bg_color: Color32::from_gray(10), // e.g. TextEdit background
            code_bg_color: Color32::from_gray(64),
            window_rounding: Rounding::same(0.0),
            window_shadow: Shadow {
                extrusion: 0.0,
                color: Color32::from_black_alpha(0),
            },
            popup_shadow: Shadow {
                extrusion: 0.0,
                color: Color32::from_black_alpha(0),
            },
            resize_corner_size: 8.0,
            text_cursor_width: 2.0,
            text_cursor_preview: false,
            clip_rect_margin: 3.0, // should be at least half the size of the widest frame stroke + max WidgetVisuals::expansion
            button_frame: true,
            collapsing_header_frame: false,
        },
        animation_time: 1.0 / 12.0,
        debug: Default::default(),
        explanation_tooltips: false,
    }
}

pub(super) fn fonts() -> egui::FontDefinitions {
    use egui::{FontData, FontDefinitions, FontFamily};
    // use the default egui fonts
    let mut fonts = FontDefinitions::default();
    // add own font
    fonts.font_data.insert(
        "Jet-Brains-Mono".to_owned(),
        FontData::from_static(include_bytes!("fonts/JetBrainsMonoNL-Medium.ttf")),
    );
    // change families
    fonts.families.insert(
        FontFamily::Monospace,
        vec![
            // "Jet-Brains-Mono".to_owned(),
            "Hack".to_owned(),
            "Ubuntu-Light".to_owned(), // fallback for √ etc
            "NotoEmoji-Regular".to_owned(),
            "emoji-icon-font".to_owned(),
        ],
    );
    fonts.families.insert(
        FontFamily::Proportional,
        vec![
            "Jet-Brains-Mono".to_owned(),
            "NotoEmoji-Regular".to_owned(),
            "emoji-icon-font".to_owned(),
        ],
    );
    // change the families
    fonts
}
