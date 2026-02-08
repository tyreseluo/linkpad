use crate::i18n;
use crate::settings_store;
use crate::state::{AppState, Language, Page, ThemePreference};
use makepad_components::button::MpButtonWidgetRefExt;
use makepad_components::makepad_widgets::*;

live_design! {
    use link::widgets::*;
    use link::linkpad_theme::*;
    use makepad_components::layout::*;
    use crate::ui::dashboard::*;
    use crate::ui::header::*;
    use crate::ui::sidebar::*;

    App = {{App}} {
        ui: <Root> {
            <Window> {
                window: {inner_size: vec2(1200, 760), title: "Linkpad"},
                body = <MpLayoutBody> {
                    width: Fill,
                    height: Fill,
                    show_bg: true,
                    draw_bg: {color: (APP_BG)},

                    sidebar = <Sidebar> {}

                    content = <View> {
                        width: Fill,
                        height: Fill,
                        flow: Down,
                        spacing: (SPACE_4),

                        header = <Header> {}
                        dashboard = <Dashboard> {}
                    }
                }
            }
        }
    }
}

app_main!(App);

#[derive(Live, LiveHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    state: AppState,
}

#[derive(Clone, Copy)]
struct ThemePalette {
    app_bg: Vec4,
    sidebar_bg: Vec4,
    panel_bg: Vec4,
    panel_alt_bg: Vec4,
    border_color: Vec4,
    text_primary: Vec4,
    text_brand: Vec4,
    menu_active_bg: Vec4,
    menu_active_hover: Vec4,
    menu_active_pressed: Vec4,
    menu_inactive_bg: Vec4,
    menu_inactive_hover: Vec4,
    menu_inactive_pressed: Vec4,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_components::makepad_widgets::live_design(cx);
        cx.link(live_id!(theme), live_id!(theme_desktop_light));
        cx.link(live_id!(theme_colors), live_id!(theme_colors_light));
        makepad_components::live_design(cx);
        crate::ui::live_design(cx);
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

impl App {
    fn theme_palette(&self) -> ThemePalette {
        match self.state.theme {
            ThemePreference::Dark => ThemePalette {
                app_bg: vec4(0.059, 0.067, 0.082, 1.0),
                sidebar_bg: vec4(0.078, 0.094, 0.129, 1.0),
                panel_bg: vec4(0.106, 0.133, 0.176, 1.0),
                panel_alt_bg: vec4(0.125, 0.157, 0.220, 1.0),
                border_color: vec4(0.188, 0.235, 0.333, 1.0),
                text_primary: vec4(0.902, 0.922, 0.949, 1.0),
                text_brand: vec4(0.961, 0.969, 0.980, 1.0),
                menu_active_bg: vec4(0.176, 0.263, 0.416, 1.0),
                menu_active_hover: vec4(0.192, 0.286, 0.455, 1.0),
                menu_active_pressed: vec4(0.145, 0.220, 0.353, 1.0),
                menu_inactive_bg: vec4(0.0, 0.0, 0.0, 0.0),
                menu_inactive_hover: vec4(0.137, 0.200, 0.302, 1.0),
                menu_inactive_pressed: vec4(0.176, 0.263, 0.416, 1.0),
            },
            ThemePreference::Light | ThemePreference::System => ThemePalette {
                app_bg: vec4(0.957, 0.961, 0.969, 1.0),
                sidebar_bg: vec4(1.0, 1.0, 1.0, 1.0),
                panel_bg: vec4(1.0, 1.0, 1.0, 1.0),
                panel_alt_bg: vec4(0.973, 0.980, 0.988, 1.0),
                border_color: vec4(0.890, 0.910, 0.941, 1.0),
                text_primary: vec4(0.059, 0.090, 0.165, 1.0),
                text_brand: vec4(0.070, 0.078, 0.090, 1.0),
                menu_active_bg: vec4(0.858, 0.918, 0.996, 1.0),
                menu_active_hover: vec4(0.858, 0.918, 0.996, 1.0),
                menu_active_pressed: vec4(0.780, 0.871, 0.984, 1.0),
                menu_inactive_bg: vec4(0.0, 0.0, 0.0, 0.0),
                menu_inactive_hover: vec4(0.937, 0.965, 1.0, 1.0),
                menu_inactive_pressed: vec4(0.858, 0.918, 0.996, 1.0),
            },
        }
    }

    fn apply_state(&mut self, cx: &mut Cx) {
        let strings = i18n::strings(self.state.language);
        let is_profiles = self.state.active_page == Page::Profiles;
        let title = i18n::page_title(strings, self.state.active_page);

        self.ui.label(ids!(header.title_label)).set_text(cx, title);
        self.ui
            .view(ids!(dashboard.profiles_section))
            .set_visible(cx, is_profiles);
        self.ui
            .view(ids!(dashboard.settings_section))
            .set_visible(cx, !is_profiles);

        self.ui
            .mp_button(ids!(sidebar.menu_profiles))
            .set_text(strings.menu_profiles);
        self.ui
            .mp_button(ids!(sidebar.menu_settings))
            .set_text(strings.menu_settings);
        self.ui
            .label(ids!(sidebar.brand))
            .set_text(cx, strings.app_name);

        self.ui
            .label(ids!(dashboard.basic_setting_title))
            .set_text(cx, strings.basic_setting_title);
        self.ui
            .label(ids!(dashboard.language_label))
            .set_text(cx, strings.language_label);
        self.ui
            .label(ids!(dashboard.theme_label))
            .set_text(cx, strings.theme_label);

        let language_dropdown = self.ui.drop_down(ids!(dashboard.language_dropdown));
        language_dropdown.set_labels(cx, i18n::language_options(self.state.language));
        language_dropdown.set_selected_item(cx, self.state.language.as_index());

        let theme_dropdown = self.ui.drop_down(ids!(dashboard.theme_dropdown));
        theme_dropdown.set_labels(cx, i18n::theme_options(self.state.language));
        theme_dropdown.set_selected_item(cx, self.state.theme.as_index());

        self.update_menu_buttons(cx);
        self.ui.redraw(cx);
    }

    fn apply_theme(&mut self, cx: &mut Cx) {
        self.apply_theme_preference(cx);
        self.apply_theme_palette(cx);
    }

    fn apply_theme_preference(&self, cx: &mut Cx) {
        // TODO: detect actual system theme for `System` option.
        let dark = matches!(self.state.theme, ThemePreference::Dark);
        if dark {
            cx.link(live_id!(theme), live_id!(theme_desktop_dark));
            cx.link(live_id!(theme_colors), live_id!(theme_colors_dark));
        } else {
            cx.link(live_id!(theme), live_id!(theme_desktop_light));
            cx.link(live_id!(theme_colors), live_id!(theme_colors_light));
        }
        cx.redraw_all();
    }

    fn load_persisted_settings(&mut self) {
        if let Some((language, theme)) = settings_store::load() {
            self.state.language = language;
            self.state.theme = theme;
        }
    }

    fn persist_settings(&self) {
        let _ = settings_store::save(self.state.language, self.state.theme);
    }

    fn apply_theme_palette(&mut self, cx: &mut Cx) {
        let palette = self.theme_palette();
        self.ui.widget(ids!(body)).apply_over(
            cx,
            live! {
                draw_bg: { color: (palette.app_bg) }
            },
        );
        self.ui.widget(ids!(sidebar)).apply_over(
            cx,
            live! {
                draw_bg: { bg_color: (palette.sidebar_bg), line_color: (palette.border_color) }
            },
        );
        self.ui.widget(ids!(header)).apply_over(
            cx,
            live! {
                draw_bg: { bg_color: (palette.panel_bg), line_color: (palette.border_color) }
            },
        );
        self.ui.widget(ids!(dashboard.content_panel)).apply_over(
            cx,
            live! {
                draw_bg: { color: (palette.panel_bg) }
            },
        );
        self.ui.widget(ids!(dashboard.basic_settings_card)).apply_over(
            cx,
            live! {
                draw_bg: { color: (palette.panel_alt_bg) }
            },
        );
        self.ui.view(ids!(dashboard.profiles_section)).apply_over(
            cx,
            live! {
                draw_bg: { color: (palette.panel_alt_bg) }
            },
        );
        self.ui.label(ids!(sidebar.brand)).apply_over(
            cx,
            live! {
                draw_text: { color: (palette.text_brand) }
            },
        );
        self.ui.label(ids!(header.title_label)).apply_over(
            cx,
            live! {
                draw_text: { color: (palette.text_primary) }
            },
        );
        self.ui.label(ids!(dashboard.language_label)).apply_over(
            cx,
            live! {
                draw_text: { color: (palette.text_primary) }
            },
        );
        self.ui.label(ids!(dashboard.theme_label)).apply_over(
            cx,
            live! {
                draw_text: { color: (palette.text_primary) }
            },
        );
        self.ui.label(ids!(dashboard.basic_setting_title)).apply_over(
            cx,
            live! {
                draw_text: { color: (palette.text_primary) }
            },
        );
    }

    fn update_menu_buttons(&mut self, cx: &mut Cx) {
        let palette = self.theme_palette();

        let profiles_btn = self.ui.widget(ids!(sidebar.menu_profiles));
        let settings_btn = self.ui.widget(ids!(sidebar.menu_settings));

        if self.state.active_page == Page::Profiles {
            profiles_btn.apply_over(cx, live! {
                draw_bg: { color: (palette.menu_active_bg), color_hover: (palette.menu_active_hover), color_pressed: (palette.menu_active_pressed) }
                draw_text: { color: (palette.text_primary) }
            });
            settings_btn.apply_over(cx, live! {
                draw_bg: { color: (palette.menu_inactive_bg), color_hover: (palette.menu_inactive_hover), color_pressed: (palette.menu_inactive_pressed) }
                draw_text: { color: (palette.text_primary) }
            });
        } else {
            profiles_btn.apply_over(cx, live! {
                draw_bg: { color: (palette.menu_inactive_bg), color_hover: (palette.menu_inactive_hover), color_pressed: (palette.menu_inactive_pressed) }
                draw_text: { color: (palette.text_primary) }
            });
            settings_btn.apply_over(cx, live! {
                draw_bg: { color: (palette.menu_active_bg), color_hover: (palette.menu_active_hover), color_pressed: (palette.menu_active_pressed) }
                draw_text: { color: (palette.text_primary) }
            });
        }
    }
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        self.load_persisted_settings();
        self.apply_state(cx);
        self.apply_theme(cx);
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if self
            .ui
            .mp_button(ids!(sidebar.menu_profiles))
            .clicked(actions)
        {
            self.state.active_page = Page::Profiles;
            self.apply_state(cx);
        }
        if self
            .ui
            .mp_button(ids!(sidebar.menu_settings))
            .clicked(actions)
        {
            self.state.active_page = Page::Settings;
            self.apply_state(cx);
        }
        if let Some(index) = self
            .ui
            .drop_down(ids!(dashboard.language_dropdown))
            .changed(actions)
        {
            self.state.language = Language::from_index(index);
            self.persist_settings();
            self.apply_state(cx);
        }
        if let Some(index) = self
            .ui
            .drop_down(ids!(dashboard.theme_dropdown))
            .changed(actions)
        {
            self.state.theme = ThemePreference::from_index(index);
            self.persist_settings();
            self.apply_state(cx);
            self.apply_theme(cx);
        }
    }
}
