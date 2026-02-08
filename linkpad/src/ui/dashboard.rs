use makepad_components::makepad_widgets::*;

live_design! {
    use link::widgets::*;
    use link::linkpad_theme::*;
    use makepad_components::card::*;
    use makepad_components::dropdown::*;
    use makepad_components::layout::*;

    pub Dashboard = <View> {
        width: Fill,
        height: Fill,
        flow: Down,
        spacing: (SPACE_4),
        padding: (SPACE_4),

        content_panel = <MpLayoutContent> {
            width: Fill,
            height: Fill,
            padding: (SPACE_4),
            flow: Down,
            spacing: (SPACE_3),
            draw_bg: {color: (PANEL_BG)},

            content_body = <View> {
                width: Fill,
                height: Fill,
                flow: Down,
                spacing: (SPACE_3),

                profiles_section = <View> {
                    width: Fill,
                    height: Fill,
                    show_bg: true,
                    draw_bg: {color: (PANEL_ALT_BG)}
                }

                settings_section = <View> {
                    width: Fill,
                    height: Fit,
                    flow: Down,
                    spacing: (SPACE_3),

                    basic_settings_card = <MpCard> {
                        width: Fill,
                        <MpCardHeader> {
                            basic_setting_title = <MpCardTitle> { text: "Linkpad Basic Setting" }
                        }

                        <MpCardContent> {
                            width: Fill,
                            flow: Down,
                            spacing: (SPACE_3),

                            <View> {
                                width: Fill,
                                height: Fit,
                                flow: Right,
                                align: {y: 0.5},
                                spacing: (SPACE_3),

                                language_label = <Label> {text: "Language", draw_text: {text_style: <APP_FONT_BODY>{}, color: (TEXT_PRIMARY)}}
                                <View> {width: Fill, height: Fit}
                                language_dropdown = <MpDropdown> {
                                    width: 200,
                                    labels: ["English", "简体中文"],
                                    selected_item: 0
                                }
                            }

                            <View> {
                                width: Fill,
                                height: Fit,
                                flow: Right,
                                align: {y: 0.5},
                                spacing: (SPACE_3),

                                theme_label = <Label> {text: "Theme", draw_text: {text_style: <APP_FONT_BODY>{}, color: (TEXT_PRIMARY)}}
                                <View> {width: Fill, height: Fit}
                                theme_dropdown = <MpDropdown> {
                                    width: 200,
                                    labels: ["Light", "Dark", "System"],
                                    selected_item: 2
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
