use makepad_components::makepad_widgets::*;
use makepad_components::button::MpButtonWidgetRefExt;
use crate::state::{AppState, Page};

live_design! {
    use link::widgets::*;
    use link::linkpad_theme::*;
    use crate::ui::dashboard::*;
    use crate::ui::header::*;
    use crate::ui::sidebar::*;

    App = {{App}} {
        ui: <Root> {
            <Window> {
                window: {inner_size: vec2(1200, 760), title: "Linkpad"},
                body = <View> {
                    width: Fill,
                    height: Fill,
                    flow: Right,
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

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_components::makepad_widgets::live_design(cx);
        cx.link(live_id!(theme), live_id!(theme_desktop_light));
        cx.link(live_id!(theme_colors), live_id!(theme_colors_light));
        makepad_components::live_design(cx);
        crate::ui::live_design(cx);
        cx.link(live_id!(linkpad_theme), live_id!(linkpad_theme_default));
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

impl App {
    fn apply_state(&mut self, cx: &mut Cx) {
        let content = self.state.active_content();
        self.ui.label(ids!(header.title_label)).set_text(cx, &content.title);
        self.ui.label(ids!(dashboard.page_title)).set_text(cx, &content.title);
        self.ui.label(ids!(dashboard.page_desc)).set_text(cx, &content.description);

        self.ui
            .mp_button(ids!(sidebar.menu_profiles))
            .set_text(&self.state.menu.profiles_title);
        self.ui
            .mp_button(ids!(sidebar.menu_settings))
            .set_text(&self.state.menu.settings_title);

        self.update_menu_buttons(cx);
        self.ui.redraw(cx);
    }

    fn update_menu_buttons(&mut self, cx: &mut Cx) {
        let active_bg = vec4(0.858, 0.918, 0.996, 1.0);
        let active_hover = vec4(0.858, 0.918, 0.996, 1.0);
        let active_pressed = vec4(0.780, 0.871, 0.984, 1.0);
        let active_text = vec4(0.059, 0.090, 0.165, 1.0);

        let inactive_bg = vec4(0.0, 0.0, 0.0, 0.0);
        let inactive_hover = vec4(0.937, 0.965, 1.0, 1.0);
        let inactive_pressed = vec4(0.858, 0.918, 0.996, 1.0);
        let inactive_text = vec4(0.059, 0.090, 0.165, 1.0);

        let profiles_btn = self.ui.widget(ids!(sidebar.menu_profiles));
        let settings_btn = self.ui.widget(ids!(sidebar.menu_settings));

        if self.state.active_page == Page::Profiles {
            profiles_btn.apply_over(cx, live! {
                draw_bg: { color: (active_bg), color_hover: (active_hover), color_pressed: (active_pressed) }
                draw_text: { color: (active_text) }
            });
            settings_btn.apply_over(cx, live! {
                draw_bg: { color: (inactive_bg), color_hover: (inactive_hover), color_pressed: (inactive_pressed) }
                draw_text: { color: (inactive_text) }
            });
        } else {
            profiles_btn.apply_over(cx, live! {
                draw_bg: { color: (inactive_bg), color_hover: (inactive_hover), color_pressed: (inactive_pressed) }
                draw_text: { color: (inactive_text) }
            });
            settings_btn.apply_over(cx, live! {
                draw_bg: { color: (active_bg), color_hover: (active_hover), color_pressed: (active_pressed) }
                draw_text: { color: (active_text) }
            });
        }
    }
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        self.apply_state(cx);
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.ui.mp_button(ids!(sidebar.menu_profiles)).clicked(actions) {
            self.state.active_page = Page::Profiles;
            self.apply_state(cx);
        }
        if self.ui.mp_button(ids!(sidebar.menu_settings)).clicked(actions) {
            self.state.active_page = Page::Settings;
            self.apply_state(cx);
        }
    }
}
