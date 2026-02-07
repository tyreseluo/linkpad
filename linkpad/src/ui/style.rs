use makepad_components::makepad_widgets::*;

live_design! {
    link linkpad_theme_default;
    use link::theme::*;

    // Colors
    pub APP_BG = #xf4f5f7;
    pub SIDEBAR_BG = #xffffff;
    pub PANEL_BG = #xffffff;
    pub PANEL_ALT_BG = #xf8fafc;
    pub PANEL_ACCENT_BG = #xeef2f7;
    pub MENU_ACTIVE_BG = #xdbeafe;
    pub MENU_HOVER_BG = #xeff6ff;
    pub TEXT_PRIMARY = #x111827;
    pub TEXT_MUTED = #x6b7280;
    pub TEXT_BRAND = #x121417;
    pub STATUS_WARN = #xf59e0b;
    pub TRANSPARENT = #x00000000;

    // Spacing
    pub SPACE_1 = 4.0;
    pub SPACE_2 = 8.0;
    pub SPACE_3 = 12.0;
    pub SPACE_4 = 16.0;
    pub SPACE_5 = 20.0;
    pub SPACE_6 = 24.0;
    pub SIDEBAR_WIDTH = 220.0;

    // Typography
    pub APP_FONT_TITLE = <THEME_FONT_BOLD>{ font_size: 16.0 };
    pub APP_FONT_BODY = <THEME_FONT_REGULAR>{ font_size: 14.0 };
    pub APP_FONT_CAPTION = <THEME_FONT_REGULAR>{ font_size: 12.0 };
}
