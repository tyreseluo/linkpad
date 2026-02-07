use makepad_components::makepad_widgets::*;

live_design! {
    use link::widgets::*;
    use link::linkpad_theme::*;
    use makepad_components::button::*;

    MenuButton = <MpButton> {
        width: Fill,
        height: 40,
        align: {x: 0.0, y: 0.5},
        padding: {left: (SPACE_3), right: (SPACE_3), top: (SPACE_2), bottom: (SPACE_2)},
        draw_bg: {
            color: (TRANSPARENT),
            color_hover: (MENU_HOVER_BG),
            color_pressed: (MENU_ACTIVE_BG),
            border_width: 0.0,
            border_color: (TRANSPARENT)
        }
        draw_text: {
            text_style: <APP_FONT_BODY>{},
            color: (TEXT_PRIMARY)
        }
    }

    pub Sidebar = <View> {
        width: (SIDEBAR_WIDTH),
        height: Fill,
        flow: Down,
        spacing: (SPACE_4),
        padding: (SPACE_4),
        show_bg: true,
        draw_bg: {color: (SIDEBAR_BG)},

        brand = <Label> {
            text: "Linkpad",
            draw_text: {text_style: <APP_FONT_TITLE>{}, color: (TEXT_BRAND)}
        }

        menu = <View> {
            width: Fill,
            height: Fit,
            flow: Down,
            spacing: (SPACE_2),

            menu_profiles = <MenuButton> { text: "Profiles" }
            menu_settings = <MenuButton> { text: "Settings" }
        }
    }
}
