use makepad_components::makepad_widgets::*;

live_design! {
    use link::widgets::*;
    use crate::ui::style::*;
    use makepad_components::layout::*;

    pub Header = <MpLayoutHeader> {
        width: Fill,
        height: 56,
        padding: {left: (SPACE_4), right: (SPACE_4)},
        flow: Right,
        align: {y: 0.5},
        spacing: (SPACE_3),
        draw_bg: {
            bg_color: (PANEL_BG),
            line_width: 0.0
        },

        title_label = <Label> {
            text: "Profiles",
            draw_text: {text_style: <APP_FONT_TITLE>{}, color: (TEXT_PRIMARY)},
        }
    }
}
