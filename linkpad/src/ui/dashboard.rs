use makepad_components::makepad_widgets::*;

live_design! {
    use link::widgets::*;
    use link::linkpad_theme::*;

    pub Dashboard = <View> {
        width: Fill,
        height: Fill,
        flow: Down,
        spacing: (SPACE_4),
        padding: (SPACE_4),

        content_panel = <View> {
            width: Fill,
            height: Fill,
            padding: (SPACE_4),
            flow: Down,
            spacing: (SPACE_3),
            show_bg: true,
            draw_bg: {color: (PANEL_BG)},

            page_title = <Label> {text: "Profiles", draw_text: {text_style: <APP_FONT_BODY>{}, color: (TEXT_PRIMARY)}}
            page_desc = <Label> {text: "Manage profiles here.", draw_text: {text_style: <APP_FONT_CAPTION>{}, color: (TEXT_MUTED)}}
            content_body = <View> {width: Fill, height: Fill, show_bg: true, draw_bg: {color: (PANEL_ALT_BG)}}
        }
    }
}
