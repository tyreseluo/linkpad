use crate::i18n;
use crate::state::{
    AppState, Language, Page, ProfileSummary, ProxyGroupSummary, ProxyNodeSummary, RuleFilter,
    ThemePreference,
};
use crate::store::profile_store;
use crate::store::settings_store;
use linkpad_core::{Core as LinkpadCore, KernelUpgradeInfo, ProxyMode};
use makepad_components::button::MpButtonWidgetRefExt;
use makepad_components::makepad_widgets::makepad_platform::CxOsOp;
use makepad_components::makepad_widgets::*;
use makepad_components::switch::MpSwitchWidgetRefExt;
use std::collections::HashMap;
use std::sync::Once;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::thread;
use tracing::{error, info, warn};

#[path = "views/profiles.rs"]
mod profiles;
#[path = "views/proxy_groups.rs"]
mod proxy_groups;
#[path = "views/rules.rs"]
mod rules;
#[path = "views/settings.rs"]
mod settings;
#[path = "tray.rs"]
mod tray;

live_design! {
    use link::widgets::*;
    use crate::ui::style::*;
    use makepad_components::layout::*;
    use crate::ui::dashboard::*;
    use crate::ui::header::*;
    use crate::ui::sidebar::*;

    App = {{App}} {
        ui: <Root> {
            <Window> {
                window: { inner_size: vec2(1280, 800), title: "Linkpad" },
                pass: { clear_color: #FFFFFF00 }
                caption_bar = {
                    caption_label = {
                        label = {
                            margin: {left: 65},
                            align: {x: 0.5},
                            text: "Linkpad",
                            draw_text: {color: (TEXT_PRIMARY)}
                        }
                    }
                    windows_buttons = {
                        // Note: these are the background colors of the buttons used in Windows:
                        // * idle: Clear, for all three buttons.
                        // * hover: #E9E9E9 for minimize and maximize, #E81123 for close.
                        // * down: either darker (on light mode) or lighter (on dark mode).
                        //
                        // However, the DesktopButton widget doesn't support drawing a background color yet,
                        // so these colors are the colors of the icon itself, not the background highlight.
                        // When it supports that, we will keep the icon color always black,
                        // and change the background color instead based on the above colors.
                        min   = { draw_bg: {color: #0, color_hover: #9, color_down: #3} }
                        max   = { draw_bg: {color: #0, color_hover: #9, color_down: #3} }
                        close = { draw_bg: {color: #0, color_hover: #E81123, color_down: #FF0015} }
                    }
                    draw_bg: {color: #F3F3F3},
                }
                body = <View> {
                    width: Fill,
                    height: Fill,
                    flow: Overlay,
                    show_bg: true,
                    draw_bg: {color: (APP_BG)},

                    main_layout = <MpLayoutBody> {
                        width: Fill,
                        height: Fill,

                        sidebar = <Sidebar> {}

                        content = <View> {
                            width: Fill,
                            height: Fill,
                            flow: Down,
                            spacing: 0,

                            header = <Header> {}
                            dashboard = <Dashboard> {
                                width: Fill,
                                height: Fill
                            }
                        }
                    }

                    toasts = <View> {
                        width: Fill,
                        height: Fill,
                        flow: Down,
                        align: {x: 1.0, y: 0.0},
                        padding: {left: 0.0, right: (SPACE_4), top: (SPACE_4), bottom: 0.0},

                        toast = <View> {
                            width: 360,
                            height: Fit,
                            flow: Down,
                            spacing: (SPACE_1),
                            padding: {left: (SPACE_3), right: (SPACE_3), top: (SPACE_3), bottom: (SPACE_3)},
                            show_bg: true,
                            draw_bg: {color: (PANEL_ALT_BG)},

                            toast_title = <Label> {
                                text: "Success"
                                draw_text: {text_style: <APP_FONT_BODY>{}, color: (TEXT_PRIMARY)}
                            }
                            toast_message = <Label> {
                                width: 312,
                                text: ""
                                draw_text: {text_style: <APP_FONT_CAPTION>{}, wrap: Word, color: (TEXT_MUTED)}
                            }
                        }
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
    #[rust]
    saved_proxy_group_selections: HashMap<String, String>,
    #[rust]
    core: LinkpadCore,
    #[rust]
    import_rx: Option<Receiver<ImportTaskResult>>,
    #[rust]
    import_poll_timer: Timer,
    #[rust]
    latency_rx: Option<Receiver<LatencyTaskEvent>>,
    #[rust]
    latency_poll_timer: Timer,
    #[rust]
    proxy_latency_ms: HashMap<String, LatencyStatus>,
    #[rust]
    latency_testing_group: Option<String>,
    #[rust]
    pending_locate: Option<(usize, usize)>,
    #[rust]
    locate_retry_count: usize,
    #[rust]
    locate_timer: Timer,
    #[rust]
    notification_queue: Vec<Notification>,
    #[rust]
    active_notification: Option<Notification>,
    #[rust]
    notification_timer: Timer,
    #[rust]
    core_task_rx: Option<Receiver<CoreTaskResult>>,
    #[rust]
    core_task_timer: Timer,
    #[rust]
    core_task_kind: Option<CoreTaskKind>,
    #[rust]
    tray: Option<makepad_components::shell::TrayHandle>,
    #[rust]
    app_menu_installed: bool,
    #[rust]
    window_id: Option<WindowId>,
    #[rust]
    window_focused: bool,
    #[rust]
    silent_start_requested: bool,
    #[rust]
    silent_start_applied: bool,
}

#[derive(Clone, Copy)]
struct ThemePalette {
    app_bg: Vec4,
    sidebar_bg: Vec4,
    panel_bg: Vec4,
    panel_alt_bg: Vec4,
    panel_accent_bg: Vec4,
    border_color: Vec4,
    text_primary: Vec4,
    text_muted: Vec4,
    text_brand: Vec4,
    status_success: Vec4,
    status_error: Vec4,
    menu_active_bg: Vec4,
    menu_active_hover: Vec4,
    menu_active_pressed: Vec4,
    menu_inactive_bg: Vec4,
    menu_inactive_hover: Vec4,
    menu_inactive_pressed: Vec4,
}

#[derive(Clone, Copy)]
struct ProxyGroupRowIds {
    row: &'static [LiveId],
    name: &'static [LiveId],
    meta: &'static [LiveId],
    status: &'static [LiveId],
    test_btn: &'static [LiveId],
    locate_btn: &'static [LiveId],
    open_btn: &'static [LiveId],
    details: &'static [LiveId],
    detail_empty: &'static [LiveId],
    detail_overflow: &'static [LiveId],
}

#[derive(Clone, Copy)]
struct ProxyItemRowIds {
    row: [LiveId; 6],
    name: [LiveId; 7],
    meta: [LiveId; 7],
    speed: [LiveId; 7],
    select_btn: [LiveId; 7],
}

type ImportTaskResult = Result<(), String>;
type CoreTaskResult = Result<CoreTaskOutput, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LatencyStatus {
    NotTested,
    Timeout,
    Value(u32),
}

#[derive(Debug)]
enum LatencyTaskEvent {
    Progress {
        group_name: String,
        proxy_name: String,
        delay: Option<u32>,
    },
    Finished {
        group_name: String,
        success_count: usize,
        total_count: usize,
    },
    Failed {
        group_name: String,
        error: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CoreTaskKind {
    Upgrading,
    Restarting,
}

#[derive(Debug)]
enum CoreTaskOutput {
    Upgraded(KernelUpgradeInfo),
    Restarted,
}

#[derive(Clone)]
struct Notification {
    level: NotificationLevel,
    message: String,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum NotificationLevel {
    Success,
    Error,
    Info,
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
        if let Event::WindowGeomChange(ev) = event {
            self.window_id = Some(ev.window_id);
        }
        if let Event::WindowGotFocus(window_id) = event {
            self.window_id = Some(*window_id);
            self.window_focused = true;
        }
        if let Event::WindowLostFocus(window_id) = event {
            if self.window_id == Some(*window_id) {
                self.window_focused = false;
            }
        }
        if let Event::WindowCloseRequested(ev) = event {
            self.window_id = Some(ev.window_id);
            if self.state.close_to_tray_enabled {
                ev.accept_close.set(false);
                self.window_focused = false;
                #[cfg(target_os = "windows")]
                {
                    cx.push_unique_platform_op(CxOsOp::MinimizeWindow(ev.window_id));
                }
                #[cfg(not(target_os = "windows"))]
                {
                    cx.push_unique_platform_op(CxOsOp::HideWindow(ev.window_id));
                }
            }
        }
        self.apply_silent_start_visibility(cx);
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
        self.try_lazy_load_rules_on_scroll(cx, event);
    }
}

impl App {
    fn proxy_group_rows() -> [ProxyGroupRowIds; 8] {
        [
            ProxyGroupRowIds {
                row: ids!(dashboard.proxy_group_row_1),
                name: ids!(dashboard.proxy_group_row_1.header.group_name),
                meta: ids!(dashboard.proxy_group_row_1.header.group_meta),
                status: ids!(dashboard.proxy_group_row_1.header.group_status),
                test_btn: ids!(dashboard.proxy_group_row_1.header.group_test_btn),
                locate_btn: ids!(dashboard.proxy_group_row_1.header.group_locate_btn),
                open_btn: ids!(dashboard.proxy_group_row_1.header.group_open_btn),
                details: ids!(dashboard.proxy_group_row_1.details),
                detail_empty: ids!(dashboard.proxy_group_row_1.details.detail_empty),
                detail_overflow: ids!(dashboard.proxy_group_row_1.details.detail_overflow),
            },
            ProxyGroupRowIds {
                row: ids!(dashboard.proxy_group_row_2),
                name: ids!(dashboard.proxy_group_row_2.header.group_name),
                meta: ids!(dashboard.proxy_group_row_2.header.group_meta),
                status: ids!(dashboard.proxy_group_row_2.header.group_status),
                test_btn: ids!(dashboard.proxy_group_row_2.header.group_test_btn),
                locate_btn: ids!(dashboard.proxy_group_row_2.header.group_locate_btn),
                open_btn: ids!(dashboard.proxy_group_row_2.header.group_open_btn),
                details: ids!(dashboard.proxy_group_row_2.details),
                detail_empty: ids!(dashboard.proxy_group_row_2.details.detail_empty),
                detail_overflow: ids!(dashboard.proxy_group_row_2.details.detail_overflow),
            },
            ProxyGroupRowIds {
                row: ids!(dashboard.proxy_group_row_3),
                name: ids!(dashboard.proxy_group_row_3.header.group_name),
                meta: ids!(dashboard.proxy_group_row_3.header.group_meta),
                status: ids!(dashboard.proxy_group_row_3.header.group_status),
                test_btn: ids!(dashboard.proxy_group_row_3.header.group_test_btn),
                locate_btn: ids!(dashboard.proxy_group_row_3.header.group_locate_btn),
                open_btn: ids!(dashboard.proxy_group_row_3.header.group_open_btn),
                details: ids!(dashboard.proxy_group_row_3.details),
                detail_empty: ids!(dashboard.proxy_group_row_3.details.detail_empty),
                detail_overflow: ids!(dashboard.proxy_group_row_3.details.detail_overflow),
            },
            ProxyGroupRowIds {
                row: ids!(dashboard.proxy_group_row_4),
                name: ids!(dashboard.proxy_group_row_4.header.group_name),
                meta: ids!(dashboard.proxy_group_row_4.header.group_meta),
                status: ids!(dashboard.proxy_group_row_4.header.group_status),
                test_btn: ids!(dashboard.proxy_group_row_4.header.group_test_btn),
                locate_btn: ids!(dashboard.proxy_group_row_4.header.group_locate_btn),
                open_btn: ids!(dashboard.proxy_group_row_4.header.group_open_btn),
                details: ids!(dashboard.proxy_group_row_4.details),
                detail_empty: ids!(dashboard.proxy_group_row_4.details.detail_empty),
                detail_overflow: ids!(dashboard.proxy_group_row_4.details.detail_overflow),
            },
            ProxyGroupRowIds {
                row: ids!(dashboard.proxy_group_row_5),
                name: ids!(dashboard.proxy_group_row_5.header.group_name),
                meta: ids!(dashboard.proxy_group_row_5.header.group_meta),
                status: ids!(dashboard.proxy_group_row_5.header.group_status),
                test_btn: ids!(dashboard.proxy_group_row_5.header.group_test_btn),
                locate_btn: ids!(dashboard.proxy_group_row_5.header.group_locate_btn),
                open_btn: ids!(dashboard.proxy_group_row_5.header.group_open_btn),
                details: ids!(dashboard.proxy_group_row_5.details),
                detail_empty: ids!(dashboard.proxy_group_row_5.details.detail_empty),
                detail_overflow: ids!(dashboard.proxy_group_row_5.details.detail_overflow),
            },
            ProxyGroupRowIds {
                row: ids!(dashboard.proxy_group_row_6),
                name: ids!(dashboard.proxy_group_row_6.header.group_name),
                meta: ids!(dashboard.proxy_group_row_6.header.group_meta),
                status: ids!(dashboard.proxy_group_row_6.header.group_status),
                test_btn: ids!(dashboard.proxy_group_row_6.header.group_test_btn),
                locate_btn: ids!(dashboard.proxy_group_row_6.header.group_locate_btn),
                open_btn: ids!(dashboard.proxy_group_row_6.header.group_open_btn),
                details: ids!(dashboard.proxy_group_row_6.details),
                detail_empty: ids!(dashboard.proxy_group_row_6.details.detail_empty),
                detail_overflow: ids!(dashboard.proxy_group_row_6.details.detail_overflow),
            },
            ProxyGroupRowIds {
                row: ids!(dashboard.proxy_group_row_7),
                name: ids!(dashboard.proxy_group_row_7.header.group_name),
                meta: ids!(dashboard.proxy_group_row_7.header.group_meta),
                status: ids!(dashboard.proxy_group_row_7.header.group_status),
                test_btn: ids!(dashboard.proxy_group_row_7.header.group_test_btn),
                locate_btn: ids!(dashboard.proxy_group_row_7.header.group_locate_btn),
                open_btn: ids!(dashboard.proxy_group_row_7.header.group_open_btn),
                details: ids!(dashboard.proxy_group_row_7.details),
                detail_empty: ids!(dashboard.proxy_group_row_7.details.detail_empty),
                detail_overflow: ids!(dashboard.proxy_group_row_7.details.detail_overflow),
            },
            ProxyGroupRowIds {
                row: ids!(dashboard.proxy_group_row_8),
                name: ids!(dashboard.proxy_group_row_8.header.group_name),
                meta: ids!(dashboard.proxy_group_row_8.header.group_meta),
                status: ids!(dashboard.proxy_group_row_8.header.group_status),
                test_btn: ids!(dashboard.proxy_group_row_8.header.group_test_btn),
                locate_btn: ids!(dashboard.proxy_group_row_8.header.group_locate_btn),
                open_btn: ids!(dashboard.proxy_group_row_8.header.group_open_btn),
                details: ids!(dashboard.proxy_group_row_8.details),
                detail_empty: ids!(dashboard.proxy_group_row_8.details.detail_empty),
                detail_overflow: ids!(dashboard.proxy_group_row_8.details.detail_overflow),
            },
        ]
    }

    const MAX_PROXY_OPTIONS_PER_GROUP: usize = 128;
    const RULES_PAGE_SIZE: usize = 50;

    fn init_logging() {
        static LOGGING_INIT: Once = Once::new();
        LOGGING_INIT.call_once(|| {
            let env_filter =
                tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                    tracing_subscriber::EnvFilter::new("linkpad=info,linkpad_core=info,warn")
                });
            if let Err(init_error) = tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .with_target(true)
                .with_thread_ids(true)
                .try_init()
            {
                eprintln!("failed to initialize tracing subscriber: {init_error}");
            }
        });
    }

    fn startup_silent_start_requested() -> bool {
        std::env::args().any(|arg| arg.eq_ignore_ascii_case("--silent-start"))
    }

    fn apply_silent_start_visibility(&mut self, cx: &mut Cx) {
        if !self.silent_start_requested || self.silent_start_applied {
            return;
        }
        let Some(window_id) = self.window_id else {
            return;
        };

        self.window_focused = false;
        #[cfg(target_os = "windows")]
        {
            cx.push_unique_platform_op(CxOsOp::MinimizeWindow(window_id));
        }
        #[cfg(not(target_os = "windows"))]
        {
            cx.push_unique_platform_op(CxOsOp::HideWindow(window_id));
        }
        self.silent_start_applied = true;
    }

    fn warmup_core_runtime_on_startup(&mut self) {
        #[cfg(target_os = "windows")]
        {
            if self.core.is_running() {
                return;
            }
            if self.core.active_profile().is_none() {
                info!("skip startup kernel warmup: no active profile");
                return;
            }
            if let Err(error) = self.core.start() {
                warn!("startup kernel warmup failed: {error}");
            } else {
                info!("startup kernel warmup succeeded");
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = self;
        }
    }

    fn proxy_item_row_ids(row_index: usize, item_index: usize) -> ProxyItemRowIds {
        let group_id = LiveId::from_str(&format!("proxy_group_row_{}", row_index + 1));
        let option_id = LiveId::from_str(&format!("option_{}", (item_index / 2) + 1));
        let column_id = if item_index % 2 == 0 {
            live_id!(left_col)
        } else {
            live_id!(right_col)
        };
        ProxyItemRowIds {
            row: [
                live_id!(dashboard),
                group_id,
                live_id!(details),
                live_id!(options_grid),
                column_id,
                option_id,
            ],
            name: [
                live_id!(dashboard),
                group_id,
                live_id!(details),
                live_id!(options_grid),
                column_id,
                option_id,
                live_id!(proxy_name),
            ],
            meta: [
                live_id!(dashboard),
                group_id,
                live_id!(details),
                live_id!(options_grid),
                column_id,
                option_id,
                live_id!(proxy_meta),
            ],
            speed: [
                live_id!(dashboard),
                group_id,
                live_id!(details),
                live_id!(options_grid),
                column_id,
                option_id,
                live_id!(proxy_speed),
            ],
            select_btn: [
                live_id!(dashboard),
                group_id,
                live_id!(details),
                live_id!(options_grid),
                column_id,
                option_id,
                live_id!(proxy_select_btn),
            ],
        }
    }

    fn theme_palette(&self) -> ThemePalette {
        match self.state.theme {
            ThemePreference::Dark => ThemePalette {
                app_bg: vec4(0.059, 0.067, 0.082, 1.0),
                sidebar_bg: vec4(0.078, 0.094, 0.129, 1.0),
                panel_bg: vec4(0.106, 0.133, 0.176, 1.0),
                panel_alt_bg: vec4(0.125, 0.157, 0.220, 1.0),
                panel_accent_bg: vec4(0.149, 0.188, 0.259, 1.0),
                border_color: vec4(0.188, 0.235, 0.333, 1.0),
                text_primary: vec4(0.902, 0.922, 0.949, 1.0),
                text_muted: vec4(0.635, 0.686, 0.761, 1.0),
                text_brand: vec4(0.961, 0.969, 0.980, 1.0),
                status_success: vec4(0.396, 0.855, 0.525, 1.0),
                status_error: vec4(1.000, 0.486, 0.486, 1.0),
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
                panel_accent_bg: vec4(0.933, 0.949, 0.973, 1.0),
                border_color: vec4(0.890, 0.910, 0.941, 1.0),
                text_primary: vec4(0.059, 0.090, 0.165, 1.0),
                text_muted: vec4(0.420, 0.447, 0.502, 1.0),
                text_brand: vec4(0.070, 0.078, 0.090, 1.0),
                status_success: vec4(0.039, 0.647, 0.322, 1.0),
                status_error: vec4(0.847, 0.192, 0.192, 1.0),
                menu_active_bg: vec4(0.858, 0.918, 0.996, 1.0),
                menu_active_hover: vec4(0.858, 0.918, 0.996, 1.0),
                menu_active_pressed: vec4(0.780, 0.871, 0.984, 1.0),
                menu_inactive_bg: vec4(0.0, 0.0, 0.0, 0.0),
                menu_inactive_hover: vec4(0.937, 0.965, 1.0, 1.0),
                menu_inactive_pressed: vec4(0.858, 0.918, 0.996, 1.0),
            },
        }
    }

    fn refresh_ui(&mut self, cx: &mut Cx) {
        self.apply_theme_preference(cx);
        self.apply_theme_palette(cx);
        self.apply_state(cx);
    }

    fn apply_state(&mut self, cx: &mut Cx) {
        let strings = i18n::strings(self.state.language);
        let is_profiles = self.state.active_page == Page::Profiles;
        let is_proxy_groups = self.state.active_page == Page::ProxyGroups;
        let is_rules = self.state.active_page == Page::Rules;
        let is_settings = self.state.active_page == Page::Settings;
        let title = i18n::page_title(strings, self.state.active_page);

        self.ui.label(ids!(header.title_label)).set_text(cx, title);
        self.ui
            .view(ids!(dashboard.profiles_section))
            .set_visible(cx, is_profiles);
        self.ui
            .view(ids!(dashboard.proxy_groups_section))
            .set_visible(cx, is_proxy_groups);
        self.ui
            .view(ids!(dashboard.rules_section))
            .set_visible(cx, is_rules);
        self.ui
            .view(ids!(dashboard.settings_section))
            .set_visible(cx, is_settings);

        self.apply_profiles_state(cx, strings);
        self.apply_proxy_groups_state(cx, strings);
        self.apply_rules_state(cx, strings);
        self.apply_settings_state(cx, strings);
        self.apply_notification_state(cx, strings);

        self.ui
            .mp_button(ids!(sidebar.menu_profiles))
            .set_text(strings.menu_profiles);
        self.ui
            .mp_button(ids!(sidebar.menu_proxy_groups))
            .set_text(strings.menu_proxy_groups);
        self.ui
            .mp_button(ids!(sidebar.menu_rules))
            .set_text(strings.menu_rules);
        self.ui
            .mp_button(ids!(sidebar.menu_settings))
            .set_text(strings.menu_settings);
        self.ui
            .label(ids!(sidebar.brand))
            .set_text(cx, strings.app_name);

        self.sync_tray_menu(strings);
        self.update_menu_buttons(cx);
        self.ui.redraw(cx);
    }

    fn apply_settings_state(&mut self, cx: &mut Cx, strings: &i18n::Strings) {
        self.ui
            .label(ids!(dashboard.basic_setting_title))
            .set_text(cx, strings.basic_setting_title);
        self.ui
            .label(ids!(dashboard.system_setting_title))
            .set_text(cx, strings.system_setting_title);
        self.ui
            .label(ids!(dashboard.clash_setting_title))
            .set_text(cx, strings.clash_setting_title);
        self.ui
            .label(ids!(dashboard.language_label))
            .set_text(cx, strings.language_label);
        self.ui
            .label(ids!(dashboard.theme_label))
            .set_text(cx, strings.theme_label);
        self.ui
            .label(ids!(dashboard.close_to_tray_label))
            .set_text(cx, strings.close_to_tray_label);
        self.ui
            .label(ids!(dashboard.system_proxy_label))
            .set_text(cx, strings.system_proxy_label);
        self.ui
            .label(ids!(dashboard.auto_launch_label))
            .set_text(cx, strings.auto_launch_label);
        self.ui
            .label(ids!(dashboard.silent_start_label))
            .set_text(cx, strings.silent_start_label);
        self.ui
            .label(ids!(dashboard.clash_port_label))
            .set_text(cx, strings.clash_port_label);
        self.ui
            .label(ids!(dashboard.clash_core_version_label))
            .set_text(cx, strings.clash_core_version_label);
        self.ui
            .label(ids!(dashboard.clash_core_path_label))
            .set_text(cx, strings.clash_core_path_label);
        self.ui
            .mp_button(ids!(dashboard.clash_port_save_btn))
            .set_text(strings.clash_port_save_button);
        self.ui
            .mp_button(ids!(dashboard.clash_core_upgrade_btn))
            .set_text(if self.core_task_kind == Some(CoreTaskKind::Upgrading) {
                strings.clash_core_upgrading_button
            } else {
                strings.clash_core_upgrade_button
            });
        self.ui
            .mp_button(ids!(dashboard.clash_core_restart_btn))
            .set_text(if self.core_task_kind == Some(CoreTaskKind::Restarting) {
                strings.clash_core_restarting_button
            } else {
                strings.clash_core_restart_button
            });
        self.ui
            .text_input(ids!(dashboard.clash_port_input))
            .set_text(cx, &self.state.clash_port_input);
        self.ui
            .label(ids!(dashboard.clash_core_version_value))
            .set_text(cx, &self.state.clash_core_version);
        self.ui
            .label(ids!(dashboard.clash_core_path_value))
            .set_text(cx, &self.state.clash_core_path);

        let language_dropdown = self.ui.drop_down(ids!(dashboard.language_dropdown));
        language_dropdown.set_labels(cx, i18n::language_options(self.state.language));
        language_dropdown.set_selected_item(cx, self.state.language.as_index());

        let theme_dropdown = self.ui.drop_down(ids!(dashboard.theme_dropdown));
        theme_dropdown.set_labels(cx, i18n::theme_options(self.state.language));
        theme_dropdown.set_selected_item(cx, self.state.theme.as_index());

        self.ui
            .mp_switch(ids!(dashboard.system_proxy_switch))
            .set_on(cx, self.state.system_proxy_enabled);
        self.ui
            .mp_switch(ids!(dashboard.close_to_tray_switch))
            .set_on(cx, self.state.close_to_tray_enabled);
        self.ui
            .mp_switch(ids!(dashboard.auto_launch_switch))
            .set_on(cx, self.state.auto_launch_enabled);
        self.ui
            .mp_switch(ids!(dashboard.silent_start_switch))
            .set_on(cx, self.state.silent_start_enabled);
    }

    fn apply_notification_state(&mut self, cx: &mut Cx, _strings: &i18n::Strings) {
        let Some(notification) = self.active_notification.as_ref() else {
            self.ui.view(ids!(toasts.toast)).set_visible(cx, false);
            return;
        };

        self.ui.view(ids!(toasts.toast)).set_visible(cx, true);
        let palette = self.theme_palette();
        let accent_text = match notification.level {
            NotificationLevel::Success => palette.text_primary,
            NotificationLevel::Error => palette.text_primary,
            NotificationLevel::Info => palette.text_primary,
        };
        self.ui.widget(ids!(toasts.toast)).apply_over(
            cx,
            live! {
                draw_bg: { color: (palette.panel_alt_bg) }
            },
        );
        self.ui.label(ids!(toasts.toast_title)).apply_over(
            cx,
            live! {
                draw_text: { color: (accent_text) }
            },
        );
        self.ui.label(ids!(toasts.toast_message)).apply_over(
            cx,
            live! {
                draw_text: { color: (palette.text_muted) }
            },
        );

        let tint = match notification.level {
            NotificationLevel::Success => vec4(0.82, 0.95, 0.86, 1.0),
            NotificationLevel::Error => vec4(0.97, 0.86, 0.86, 1.0),
            NotificationLevel::Info => palette.panel_alt_bg,
        };
        self.ui.widget(ids!(toasts.toast)).apply_over(
            cx,
            live! {
                draw_bg: { color: (tint) }
            },
        );

        let title = match notification.level {
            NotificationLevel::Success => "Success",
            NotificationLevel::Error => "Error",
            NotificationLevel::Info => "Info",
        };
        self.ui.label(ids!(toasts.toast_title)).set_text(cx, title);
        self.ui
            .label(ids!(toasts.toast_message))
            .set_text(cx, &notification.message);
    }

    fn apply_profiles_state(&mut self, cx: &mut Cx, strings: &i18n::Strings) {
        let palette = self.theme_palette();

        self.ui
            .label(ids!(dashboard.profile_url_label))
            .set_text(cx, strings.profiles_import_url_label);
        self.ui
            .mp_button(ids!(dashboard.profile_import_btn))
            .set_text(strings.profiles_import_button);
        self.ui
            .label(ids!(dashboard.profile_import_status))
            .set_text(cx, &self.state.import_status.message);
        self.ui
            .text_input(ids!(dashboard.profile_url_input))
            .apply_over(
                cx,
                live! {
                    empty_text: (strings.profiles_import_url_placeholder)
                },
            );
        self.ui
            .text_input(ids!(dashboard.profile_url_input))
            .set_text(cx, &self.state.profile_url_input);

        let status_color = if self.state.import_status.is_error {
            palette.status_error
        } else if self.state.import_status.message == strings.profiles_import_success {
            palette.status_success
        } else {
            palette.text_muted
        };
        self.ui
            .label(ids!(dashboard.profile_import_status))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (status_color) }
                },
            );

        self.ui
            .widget(ids!(dashboard.current_profile_card))
            .set_visible(cx, false);

        self.ui
            .label(ids!(dashboard.profiles_list_title))
            .set_text(cx, strings.profiles_list_title);
        if self.state.profiles.is_empty() {
            self.ui
                .label(ids!(dashboard.profiles_empty_label))
                .set_text(cx, strings.profiles_list_empty);
        } else {
            self.ui
                .label(ids!(dashboard.profiles_empty_label))
                .set_text(cx, "");
        }

        Self::apply_profile_row_fixed(
            cx,
            &mut self.ui,
            strings,
            self.state.profiles.get(0),
            palette,
            ids!(dashboard.profile_row_1),
            ids!(dashboard.profile_row_1_name),
            ids!(dashboard.profile_row_1_meta),
            ids!(dashboard.profile_row_1_status),
            ids!(dashboard.profile_row_1_activate_btn),
            ids!(dashboard.profile_row_1_refresh_btn),
            ids!(dashboard.profile_row_1_delete_btn),
        );
        Self::apply_profile_row_fixed(
            cx,
            &mut self.ui,
            strings,
            self.state.profiles.get(1),
            palette,
            ids!(dashboard.profile_row_2),
            ids!(dashboard.profile_row_2_name),
            ids!(dashboard.profile_row_2_meta),
            ids!(dashboard.profile_row_2_status),
            ids!(dashboard.profile_row_2_activate_btn),
            ids!(dashboard.profile_row_2_refresh_btn),
            ids!(dashboard.profile_row_2_delete_btn),
        );
        Self::apply_profile_row_fixed(
            cx,
            &mut self.ui,
            strings,
            self.state.profiles.get(2),
            palette,
            ids!(dashboard.profile_row_3),
            ids!(dashboard.profile_row_3_name),
            ids!(dashboard.profile_row_3_meta),
            ids!(dashboard.profile_row_3_status),
            ids!(dashboard.profile_row_3_activate_btn),
            ids!(dashboard.profile_row_3_refresh_btn),
            ids!(dashboard.profile_row_3_delete_btn),
        );
    }

    fn apply_proxy_groups_state(&mut self, cx: &mut Cx, strings: &i18n::Strings) {
        self.ui
            .mp_button(ids!(dashboard.proxy_mode_rule_btn))
            .set_text(strings.proxy_mode_rule);
        self.ui
            .mp_button(ids!(dashboard.proxy_mode_global_btn))
            .set_text(strings.proxy_mode_global);
        self.ui
            .mp_button(ids!(dashboard.proxy_mode_direct_btn))
            .set_text(strings.proxy_mode_direct);
        self.apply_proxy_mode_buttons(cx);

        if self.state.proxy_groups.is_empty() {
            self.ui
                .label(ids!(dashboard.proxy_groups_empty))
                .set_visible(cx, true);
            self.ui
                .label(ids!(dashboard.proxy_groups_empty))
                .set_text(cx, strings.proxy_groups_empty);
            for row_ids in Self::proxy_group_rows() {
                self.ui.view(row_ids.row).set_visible(cx, false);
            }
            return;
        }
        self.ui
            .label(ids!(dashboard.proxy_groups_empty))
            .set_visible(cx, false);
        self.ui
            .label(ids!(dashboard.proxy_groups_empty))
            .set_text(cx, "");

        for (index, row_ids) in Self::proxy_group_rows().iter().enumerate() {
            self.apply_proxy_group_row_fixed(cx, strings, index, *row_ids);
        }
    }

    fn apply_rules_state(&mut self, cx: &mut Cx, strings: &i18n::Strings) {
        self.ui
            .label(ids!(dashboard.rules_title))
            .set_text(cx, strings.rules_title);
        self.ui
            .label(ids!(dashboard.rules_desc))
            .set_text(cx, strings.rules_desc);
        self.ui
            .text_input(ids!(dashboard.rules_search_input))
            .apply_over(
                cx,
                live! {
                    empty_text: (strings.rules_search_placeholder)
                },
            );
        self.ui
            .text_input(ids!(dashboard.rules_search_input))
            .set_text(cx, &self.state.rules_query);
        self.ui
            .mp_button(ids!(dashboard.rules_filter_all_btn))
            .set_text(strings.rules_filter_all);
        self.ui
            .mp_button(ids!(dashboard.rules_filter_domain_btn))
            .set_text(strings.rules_filter_domain);
        self.ui
            .mp_button(ids!(dashboard.rules_filter_ip_cidr_btn))
            .set_text(strings.rules_filter_ip_cidr);
        self.ui
            .mp_button(ids!(dashboard.rules_filter_process_btn))
            .set_text(strings.rules_filter_process_name);

        let palette = self.theme_palette();
        self.apply_menu_button_style(
            cx,
            ids!(dashboard.rules_filter_all_btn),
            self.state.rules_filter == RuleFilter::All,
            palette,
        );
        self.apply_menu_button_style(
            cx,
            ids!(dashboard.rules_filter_domain_btn),
            self.state.rules_filter == RuleFilter::Domain,
            palette,
        );
        self.apply_menu_button_style(
            cx,
            ids!(dashboard.rules_filter_ip_cidr_btn),
            self.state.rules_filter == RuleFilter::IpCidr,
            palette,
        );
        self.apply_menu_button_style(
            cx,
            ids!(dashboard.rules_filter_process_btn),
            self.state.rules_filter == RuleFilter::ProcessName,
            palette,
        );

        let filtered_rules = self.filtered_rules();
        let total_count = filtered_rules.len();
        self.ensure_rules_visible_count(total_count);
        let show_count = if self.should_paginate_rules() {
            self.state.rules_visible_count.min(total_count)
        } else {
            total_count
        };
        self.ui.label(ids!(dashboard.rules_count)).set_text(
            cx,
            &format!("{}: {}", strings.rules_count_prefix, total_count),
        );

        if total_count == 0 {
            self.ui
                .label(ids!(dashboard.rules_empty))
                .set_text(cx, strings.rules_empty);
            self.ui.label(ids!(dashboard.rules_list)).set_text(cx, "");
            return;
        }

        self.ui.label(ids!(dashboard.rules_empty)).set_text(cx, "");
        let rules_text = filtered_rules
            .iter()
            .take(show_count)
            .enumerate()
            .map(|(index, rule)| format!("{}. {}", index + 1, rule))
            .collect::<Vec<_>>()
            .join("\n");
        self.ui
            .label(ids!(dashboard.rules_list))
            .set_text(cx, &rules_text);
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_profile_row_fixed(
        cx: &mut Cx,
        ui: &mut WidgetRef,
        strings: &i18n::Strings,
        profile: Option<&ProfileSummary>,
        palette: ThemePalette,
        row_id: &[LiveId; 2],
        name_id: &[LiveId; 2],
        meta_id: &[LiveId; 2],
        status_id: &[LiveId; 2],
        activate_btn_id: &[LiveId; 2],
        refresh_btn_id: &[LiveId; 2],
        delete_btn_id: &[LiveId; 2],
    ) {
        let Some(profile) = profile else {
            ui.view(row_id).set_visible(cx, false);
            return;
        };

        ui.view(row_id).set_visible(cx, true);
        let row_bg = if profile.active {
            palette.menu_active_bg
        } else {
            palette.panel_bg
        };
        ui.view(row_id).apply_over(
            cx,
            live! {
                draw_bg: { color: (row_bg) }
            },
        );
        ui.label(name_id).set_text(cx, &profile.name);
        ui.label(meta_id).set_text(
            cx,
            &format!(
                "{}\n{}: {}",
                Self::truncate_text(&profile.source, 54),
                strings.profiles_current_updated,
                profile.updated_at
            ),
        );
        ui.label(status_id).set_text(
            cx,
            if profile.active {
                strings.profiles_status_active
            } else {
                strings.profiles_status_inactive
            },
        );
        ui.mp_button(activate_btn_id).set_text(if profile.active {
            strings.profiles_status_active
        } else {
            strings.profiles_action_activate
        });
        ui.mp_button(refresh_btn_id)
            .set_text(strings.profiles_action_refresh);
        ui.mp_button(delete_btn_id)
            .set_text(strings.profiles_action_delete);
    }

    fn apply_proxy_group_row_fixed(
        &mut self,
        cx: &mut Cx,
        strings: &i18n::Strings,
        row_index: usize,
        row_ids: ProxyGroupRowIds,
    ) {
        let Some(group) = self.state.proxy_groups.get(row_index).cloned() else {
            self.ui.view(row_ids.row).set_visible(cx, false);
            return;
        };

        self.ui.view(row_ids.row).set_visible(cx, true);
        self.ui.label(row_ids.name).set_text(cx, &group.name);
        self.ui
            .label(row_ids.meta)
            .set_text(cx, &format!("{} | {} proxies", group.kind, group.size));

        let mut selected_index = self
            .state
            .proxy_group_selected
            .get(&group.name)
            .copied()
            .unwrap_or(0);
        if !group.proxies.is_empty() && selected_index >= group.proxies.len() {
            selected_index = 0;
        }
        self.state
            .proxy_group_selected
            .insert(group.name.clone(), selected_index);
        let selected_name = group
            .proxies
            .get(selected_index)
            .cloned()
            .unwrap_or_else(|| "-".to_string());
        self.ui.label(row_ids.status).set_text(
            cx,
            &format!(
                "{}: {}",
                strings.proxy_groups_selected_prefix, selected_name
            ),
        );

        let is_open = self.state.active_proxy_group.as_deref() == Some(group.name.as_str());
        let is_latency_testing = self
            .latency_testing_group
            .as_deref()
            .map(|name| name == group.name.as_str())
            .unwrap_or(false);
        self.ui
            .mp_button(row_ids.test_btn)
            .set_text(if is_latency_testing {
                strings.proxy_groups_testing
            } else {
                strings.proxy_groups_test_latency
            });
        self.ui
            .mp_button(row_ids.locate_btn)
            .set_text(strings.proxy_groups_locate);
        self.ui.mp_button(row_ids.open_btn).set_text(if is_open {
            strings.proxy_groups_opened
        } else {
            strings.proxy_groups_open
        });

        let palette = self.theme_palette();
        let bg_color = if is_open {
            palette.menu_active_bg
        } else {
            palette.panel_accent_bg
        };
        self.ui.view(row_ids.row).apply_over(
            cx,
            live! {
                draw_bg: { color: (bg_color) }
            },
        );

        self.ui.view(row_ids.details).set_visible(cx, is_open);
        if !is_open {
            return;
        }

        if group.proxies.is_empty() {
            self.ui
                .label(row_ids.detail_empty)
                .set_text(cx, strings.proxy_groups_proxy_empty);
            self.ui.label(row_ids.detail_overflow).set_text(cx, "");
            for item_index in 0..Self::MAX_PROXY_OPTIONS_PER_GROUP {
                let item = Self::proxy_item_row_ids(row_index, item_index);
                self.ui.view(&item.row).set_visible(cx, false);
            }
            return;
        }

        self.ui.label(row_ids.detail_empty).set_text(cx, "");
        for item_index in 0..Self::MAX_PROXY_OPTIONS_PER_GROUP {
            let item = Self::proxy_item_row_ids(row_index, item_index);
            let Some(proxy_name) = group.proxies.get(item_index).cloned() else {
                self.ui.view(&item.row).set_visible(cx, false);
                continue;
            };
            self.ui.view(&item.row).set_visible(cx, true);
            let item_bg = if item_index == selected_index {
                palette.menu_active_bg
            } else {
                palette.panel_bg
            };
            self.ui.view(&item.row).apply_over(
                cx,
                live! {
                    draw_bg: { color: (item_bg) }
                },
            );
            self.ui.label(&item.name).set_text(cx, &proxy_name);
            self.ui
                .label(&item.meta)
                .set_text(cx, &self.proxy_protocol_label(strings, &proxy_name));
            let latency_state = self
                .proxy_latency_ms
                .get(&proxy_name)
                .copied()
                .unwrap_or(LatencyStatus::NotTested);
            let (latency_text, latency_color) = match latency_state {
                LatencyStatus::Value(value) => (
                    format!("{} {}", value, strings.proxy_groups_proxy_latency_suffix),
                    palette.text_muted,
                ),
                LatencyStatus::Timeout => (
                    strings.proxy_groups_latency_timeout.to_string(),
                    palette.status_error,
                ),
                LatencyStatus::NotTested => (
                    strings.proxy_groups_latency_not_tested.to_string(),
                    palette.text_muted,
                ),
            };
            self.ui.label(&item.speed).set_text(cx, &latency_text);
            self.ui.label(&item.speed).apply_over(
                cx,
                live! {
                    draw_text: { color: (latency_color) }
                },
            );
            self.ui
                .mp_button(&item.select_btn)
                .set_text(if item_index == selected_index {
                    strings.proxy_groups_proxy_selected
                } else {
                    strings.proxy_groups_proxy_use
                });
        }

        if group.proxies.len() > Self::MAX_PROXY_OPTIONS_PER_GROUP {
            self.ui.label(row_ids.detail_overflow).set_text(
                cx,
                &format!(
                    "{}: {} / {}",
                    strings.proxy_groups_proxy_overflow_prefix,
                    Self::MAX_PROXY_OPTIONS_PER_GROUP,
                    group.proxies.len()
                ),
            );
        } else {
            self.ui.label(row_ids.detail_overflow).set_text(cx, "");
        }
    }

    fn truncate_text(input: &str, max_chars: usize) -> String {
        let mut chars = input.chars();
        let truncated: String = chars.by_ref().take(max_chars).collect();
        if chars.next().is_some() {
            format!("{truncated}...")
        } else {
            truncated
        }
    }

    fn sync_from_core(&mut self) {
        self.state.profiles = self
            .core
            .profiles()
            .into_iter()
            .map(|profile| ProfileSummary {
                id: profile.id,
                name: profile.name,
                source: profile.source_url,
                updated_at: profile.updated_at,
                node_count: profile.node_count,
                group_count: profile.group_count,
                rule_count: profile.rule_count,
                active: profile.active,
            })
            .collect();

        self.state.proxy_nodes = self
            .core
            .active_proxy_nodes()
            .into_iter()
            .map(|node| ProxyNodeSummary {
                name: node.name,
                kind: node.kind,
                udp: node.udp,
            })
            .collect();
        self.proxy_latency_ms.retain(|proxy_name, _| {
            self.state
                .proxy_nodes
                .iter()
                .any(|node| node.name == *proxy_name)
        });

        self.state.proxy_groups = self
            .core
            .active_proxy_groups()
            .into_iter()
            .map(|group| ProxyGroupSummary {
                name: group.name,
                kind: group.kind,
                size: group.size,
                proxies: group.proxies,
            })
            .collect();

        self.state.rules = self.core.active_rules();
        self.reset_rules_pagination();

        self.state
            .proxy_group_selected
            .retain(|group_name, selected| {
                let Some(group) = self
                    .state
                    .proxy_groups
                    .iter()
                    .find(|group| &group.name == group_name)
                else {
                    return false;
                };
                if group.proxies.is_empty() {
                    *selected = 0;
                } else if *selected >= group.proxies.len() {
                    *selected = 0;
                }
                true
            });
        for group in &self.state.proxy_groups {
            self.state
                .proxy_group_selected
                .entry(group.name.clone())
                .or_insert(0);
        }
        let mut has_controller_selection = false;
        if let Ok(controller_selected) = self.core.current_proxy_group_selections() {
            has_controller_selection = !controller_selected.is_empty();
            for group in &self.state.proxy_groups {
                let Some(selected_proxy_name) = controller_selected.get(&group.name) else {
                    continue;
                };
                if let Some(index) = group
                    .proxies
                    .iter()
                    .position(|proxy| proxy == selected_proxy_name)
                {
                    self.state
                        .proxy_group_selected
                        .insert(group.name.clone(), index);
                }
            }
        }
        if !has_controller_selection {
            self.apply_saved_proxy_group_selections();
        }
        self.snapshot_proxy_group_selections();

        let active_exists = self
            .state
            .active_proxy_group
            .as_ref()
            .and_then(|active_name| {
                self.state
                    .proxy_groups
                    .iter()
                    .find(|group| &group.name == active_name)
            })
            .is_some();
        if !active_exists {
            self.state.active_proxy_group = self.state.proxy_groups.first().map(|g| g.name.clone());
        }

        let config = self.core.config();
        self.state.proxy_mode = config.mode;
        if let Ok(mode) = self.core.current_mode() {
            self.state.proxy_mode = mode;
        }
        self.state.clash_mixed_port = config.mixed_port;
        self.state.clash_port_input = config.mixed_port.to_string();

        let strings = i18n::strings(self.state.language);
        let kernel_info = self.core.kernel_info();
        let binary_path = kernel_info.binary_path.clone();
        self.state.clash_core_version = kernel_info.version.unwrap_or_else(|| {
            if binary_path.is_some() {
                strings.clash_core_installed_unknown_version.to_string()
            } else {
                strings.clash_core_not_found.to_string()
            }
        });
        self.state.clash_core_path = binary_path.unwrap_or(kernel_info.suggested_path);
        self.state.system_proxy_enabled = self.core.is_system_proxy_enabled();
        self.persist_settings();
    }

    fn set_proxy_mode(&mut self, cx: &mut Cx, mode: ProxyMode) {
        if self.state.proxy_mode == mode {
            return;
        }
        info!("proxy mode switch requested: {:?}", mode);

        match self.core.set_mode(mode) {
            Ok(()) => {
                self.sync_from_core();
                info!("proxy mode switched successfully");
            }
            Err(error) => {
                let strings = i18n::strings(self.state.language);
                error!("proxy mode switch failed: {error}");
                self.push_notification(
                    cx,
                    NotificationLevel::Error,
                    format!("{}: {error}", strings.proxy_mode_switch_failed_prefix),
                );
            }
        }
        self.refresh_ui(cx);
    }

    fn title_case_kind(kind: &str) -> String {
        let mut chars = kind.chars();
        match chars.next() {
            Some(first) => {
                let mut out = String::new();
                out.extend(first.to_uppercase());
                out.push_str(chars.as_str());
                out
            }
            None => "Proxy".to_string(),
        }
    }

    fn proxy_protocol_label(&self, strings: &i18n::Strings, proxy_name: &str) -> String {
        self.state
            .proxy_nodes
            .iter()
            .find(|node| node.name == proxy_name)
            .map(|node| {
                if node.udp {
                    format!(
                        "{} | {}",
                        Self::title_case_kind(&node.kind),
                        strings.proxy_groups_udp_tag
                    )
                } else {
                    Self::title_case_kind(&node.kind)
                }
            })
            .unwrap_or_else(|| strings.proxy_groups_protocol_unknown.to_string())
    }

    fn push_notification(&mut self, cx: &mut Cx, level: NotificationLevel, message: String) {
        self.notification_queue
            .push(Notification { level, message });
        self.show_next_notification(cx);
    }

    fn show_next_notification(&mut self, cx: &mut Cx) {
        if self.active_notification.is_some() || self.notification_queue.is_empty() {
            return;
        }
        self.active_notification = Some(self.notification_queue.remove(0));
        if !self.notification_timer.is_empty() {
            cx.stop_timer(self.notification_timer);
        }
        self.notification_timer = cx.start_timeout(2.5);
    }

    fn dismiss_notification(&mut self, cx: &mut Cx) {
        self.active_notification = None;
        if !self.notification_timer.is_empty() {
            cx.stop_timer(self.notification_timer);
            self.notification_timer = Timer::default();
        }
        self.show_next_notification(cx);
    }

    fn stop_core_task_polling(&mut self, cx: &mut Cx) {
        if !self.core_task_timer.is_empty() {
            cx.stop_timer(self.core_task_timer);
            self.core_task_timer = Timer::default();
        }
        self.core_task_rx = None;
        self.core_task_kind = None;
    }

    fn start_core_upgrade(&mut self, cx: &mut Cx) {
        if self.core_task_rx.is_some() {
            warn!("skip core upgrade: task already running");
            return;
        }
        info!("core upgrade requested");

        let core = self.core.clone();
        let (tx, rx) = std::sync::mpsc::channel::<CoreTaskResult>();
        thread::spawn(move || {
            let result = core
                .upgrade_kernel_binary()
                .and_then(|info| {
                    core.verify_kernel_binary()?;
                    Ok(CoreTaskOutput::Upgraded(info))
                })
                .map_err(|error| error.to_string());
            let _ = tx.send(result);
        });

        if !self.core_task_timer.is_empty() {
            cx.stop_timer(self.core_task_timer);
        }
        self.core_task_rx = Some(rx);
        self.core_task_kind = Some(CoreTaskKind::Upgrading);
        self.core_task_timer = cx.start_interval(0.1);
        self.refresh_ui(cx);
    }

    fn start_core_restart(&mut self, cx: &mut Cx) {
        if self.core_task_rx.is_some() {
            warn!("skip core restart: task already running");
            return;
        }
        info!("core restart requested");

        let core = self.core.clone();
        let (tx, rx) = std::sync::mpsc::channel::<CoreTaskResult>();
        thread::spawn(move || {
            let result = core
                .restart_kernel_runtime()
                .map(|_| CoreTaskOutput::Restarted)
                .map_err(|error| error.to_string());
            let _ = tx.send(result);
        });

        if !self.core_task_timer.is_empty() {
            cx.stop_timer(self.core_task_timer);
        }
        self.core_task_rx = Some(rx);
        self.core_task_kind = Some(CoreTaskKind::Restarting);
        self.core_task_timer = cx.start_interval(0.1);
        self.refresh_ui(cx);
    }

    fn poll_core_task(&mut self, cx: &mut Cx) {
        let Some(core_task_rx) = self.core_task_rx.as_ref() else {
            return;
        };

        let result = match core_task_rx.try_recv() {
            Ok(result) => Some(result),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(Err("core worker disconnected".to_string())),
        };
        let Some(result) = result else {
            return;
        };

        let task_kind = self.core_task_kind;
        self.stop_core_task_polling(cx);
        let strings = i18n::strings(self.state.language);
        match result {
            Ok(CoreTaskOutput::Upgraded(info)) => {
                self.sync_from_core();
                info!("core upgrade succeeded: version={}", info.version);
                self.push_notification(
                    cx,
                    NotificationLevel::Success,
                    format!(
                        "{}: {} ({})",
                        strings.clash_core_upgrade_success_prefix, info.version, info.asset_name
                    ),
                );
            }
            Ok(CoreTaskOutput::Restarted) => {
                self.sync_from_core();
                info!("core restart succeeded");
                self.push_notification(
                    cx,
                    NotificationLevel::Success,
                    strings.clash_core_restart_success.to_string(),
                );
            }
            Err(error) => {
                error!("core task failed: {error}");
                let prefix = match task_kind {
                    Some(CoreTaskKind::Upgrading) => strings.clash_core_upgrade_failed_prefix,
                    Some(CoreTaskKind::Restarting) => strings.clash_core_restart_failed_prefix,
                    None => strings.clash_core_upgrade_failed_prefix,
                };
                self.push_notification(cx, NotificationLevel::Error, format!("{prefix}: {error}"));
            }
        }
        self.refresh_ui(cx);
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

    fn apply_dropdown_theme(&mut self, cx: &mut Cx, id: &[LiveId; 2], palette: ThemePalette) {
        self.ui.widget(id).apply_over(
            cx,
            live! {
                draw_bg: {
                    color: (palette.panel_bg),
                    color_hover: (palette.panel_alt_bg),
                    color_focus: (palette.panel_bg),
                    color_down: (palette.panel_alt_bg),
                    border_color: (palette.border_color),
                    border_color_hover: (palette.border_color),
                    border_color_focus: (palette.menu_active_bg),
                    border_color_down: (palette.menu_active_bg),
                    border_color_2: (palette.border_color),
                    border_color_2_hover: (palette.border_color),
                    border_color_2_focus: (palette.menu_active_bg),
                    border_color_2_down: (palette.menu_active_bg),
                    arrow_color: (palette.text_muted),
                    arrow_color_hover: (palette.text_primary),
                    arrow_color_focus: (palette.text_primary),
                    arrow_color_down: (palette.text_primary),
                    arrow_color_disabled: (palette.text_muted)
                }
                draw_text: {
                    color: (palette.text_primary),
                    color_hover: (palette.text_primary),
                    color_focus: (palette.text_primary),
                    color_down: (palette.text_primary),
                    color_disabled: (palette.text_muted)
                }
            },
        );
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

        self.ui
            .widget(ids!(dashboard.basic_settings_card))
            .apply_over(
                cx,
                live! {
                    draw_bg: { color: (palette.panel_alt_bg) }
                },
            );
        self.ui
            .widget(ids!(dashboard.system_settings_card))
            .apply_over(
                cx,
                live! {
                    draw_bg: { color: (palette.panel_alt_bg) }
                },
            );
        self.ui
            .widget(ids!(dashboard.clash_settings_card))
            .apply_over(
                cx,
                live! {
                    draw_bg: { color: (palette.panel_alt_bg) }
                },
            );
        self.ui
            .widget(ids!(dashboard.profiles_import_card))
            .apply_over(
                cx,
                live! {
                    draw_bg: { color: (palette.panel_alt_bg) }
                },
            );
        self.ui
            .widget(ids!(dashboard.current_profile_card))
            .apply_over(
                cx,
                live! {
                    draw_bg: { color: (palette.panel_alt_bg) }
                },
            );
        self.ui
            .widget(ids!(dashboard.profiles_list_card))
            .apply_over(
                cx,
                live! {
                    draw_bg: { color: (palette.panel_alt_bg) }
                },
            );
        self.ui
            .widget(ids!(dashboard.proxy_groups_card))
            .apply_over(
                cx,
                live! {
                    draw_bg: { color: (palette.panel_alt_bg) }
                },
            );
        self.ui.widget(ids!(dashboard.rules_card)).apply_over(
            cx,
            live! {
                draw_bg: { color: (palette.panel_alt_bg) }
            },
        );

        self.ui.view(ids!(dashboard.profile_row_1)).apply_over(
            cx,
            live! {
                draw_bg: { color: (palette.panel_accent_bg) }
            },
        );
        self.ui.view(ids!(dashboard.profile_row_2)).apply_over(
            cx,
            live! {
                draw_bg: { color: (palette.panel_accent_bg) }
            },
        );
        self.ui.view(ids!(dashboard.profile_row_3)).apply_over(
            cx,
            live! {
                draw_bg: { color: (palette.panel_accent_bg) }
            },
        );
        for row in Self::proxy_group_rows() {
            self.ui.view(row.row).apply_over(
                cx,
                live! {
                    draw_bg: { color: (palette.panel_accent_bg) }
                },
            );
        }

        self.ui
            .widget(ids!(dashboard.profile_url_input))
            .apply_over(
                cx,
                live! {
                    draw_bg: {
                        bg_color: (palette.panel_bg),
                        bg_color_hover: (palette.panel_bg),
                        bg_color_focus: (palette.panel_bg),
                        border_color: (palette.border_color),
                        border_color_hover: (palette.border_color),
                        border_color_focus: (palette.menu_active_bg)
                    }
                    draw_text: {
                        color: (palette.text_primary),
                        color_empty: (palette.text_muted)
                    }
                    draw_cursor: { color: (palette.menu_active_bg) }
                },
            );
        self.apply_dropdown_theme(cx, ids!(dashboard.language_dropdown), palette);
        self.apply_dropdown_theme(cx, ids!(dashboard.theme_dropdown), palette);
        self.ui.widget(ids!(dashboard.clash_port_input)).apply_over(
            cx,
            live! {
                draw_bg: {
                    bg_color: (palette.panel_bg),
                    bg_color_hover: (palette.panel_bg),
                    bg_color_focus: (palette.panel_bg),
                    border_color: (palette.border_color),
                    border_color_hover: (palette.border_color),
                    border_color_focus: (palette.menu_active_bg)
                }
                draw_text: {
                    color: (palette.text_primary),
                    color_empty: (palette.text_muted)
                }
                draw_cursor: { color: (palette.menu_active_bg) }
            },
        );
        self.ui
            .widget(ids!(dashboard.rules_search_input))
            .apply_over(
                cx,
                live! {
                    draw_bg: {
                        bg_color: (palette.panel_bg),
                        bg_color_hover: (palette.panel_bg),
                        bg_color_focus: (palette.panel_bg),
                        border_color: (palette.border_color),
                        border_color_hover: (palette.border_color),
                        border_color_focus: (palette.menu_active_bg)
                    }
                    draw_text: {
                        color: (palette.text_primary),
                        color_empty: (palette.text_muted)
                    }
                    draw_cursor: { color: (palette.menu_active_bg) }
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

        self.ui
            .label(ids!(dashboard.basic_setting_title))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_primary) }
                },
            );
        self.ui
            .label(ids!(dashboard.system_setting_title))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_primary) }
                },
            );
        self.ui
            .label(ids!(dashboard.clash_setting_title))
            .apply_over(
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
        self.ui
            .label(ids!(dashboard.close_to_tray_label))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_primary) }
                },
            );
        self.ui
            .label(ids!(dashboard.system_proxy_label))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_primary) }
                },
            );
        self.ui.label(ids!(dashboard.auto_launch_label)).apply_over(
            cx,
            live! {
                draw_text: { color: (palette.text_primary) }
            },
        );
        self.ui
            .label(ids!(dashboard.silent_start_label))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_primary) }
                },
            );
        self.ui.label(ids!(dashboard.clash_port_label)).apply_over(
            cx,
            live! {
                draw_text: { color: (palette.text_primary) }
            },
        );
        self.ui
            .label(ids!(dashboard.clash_core_version_label))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_primary) }
                },
            );
        self.ui
            .label(ids!(dashboard.clash_core_version_value))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_muted) }
                },
            );
        self.ui
            .label(ids!(dashboard.clash_core_path_label))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_primary) }
                },
            );
        self.ui
            .label(ids!(dashboard.clash_core_path_value))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_muted) }
                },
            );

        self.ui.label(ids!(dashboard.profile_url_label)).apply_over(
            cx,
            live! {
                draw_text: { color: (palette.text_primary) }
            },
        );
        self.ui
            .label(ids!(dashboard.current_profile_title))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_primary) }
                },
            );
        self.ui
            .label(ids!(dashboard.current_profile_name))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_primary) }
                },
            );
        self.ui
            .label(ids!(dashboard.current_profile_source))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_muted) }
                },
            );
        self.ui
            .label(ids!(dashboard.current_profile_updated))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_muted) }
                },
            );
        self.ui
            .label(ids!(dashboard.current_profile_stats))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_primary) }
                },
            );
        self.ui
            .label(ids!(dashboard.current_profile_empty))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_muted) }
                },
            );
        self.ui
            .label(ids!(dashboard.profiles_list_title))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_primary) }
                },
            );
        self.ui
            .label(ids!(dashboard.profiles_empty_label))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_muted) }
                },
            );
        self.ui
            .label(ids!(dashboard.profile_row_1_name))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_primary) }
                },
            );
        self.ui
            .label(ids!(dashboard.profile_row_1_meta))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_muted) }
                },
            );
        self.ui
            .label(ids!(dashboard.profile_row_1_status))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_primary) }
                },
            );
        self.ui
            .label(ids!(dashboard.profile_row_2_name))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_primary) }
                },
            );
        self.ui
            .label(ids!(dashboard.profile_row_2_meta))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_muted) }
                },
            );
        self.ui
            .label(ids!(dashboard.profile_row_2_status))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_primary) }
                },
            );
        self.ui
            .label(ids!(dashboard.profile_row_3_name))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_primary) }
                },
            );
        self.ui
            .label(ids!(dashboard.profile_row_3_meta))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_muted) }
                },
            );
        self.ui
            .label(ids!(dashboard.profile_row_3_status))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_primary) }
                },
            );

        self.ui
            .label(ids!(dashboard.proxy_groups_empty))
            .apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_muted) }
                },
            );
        for (row_index, row) in Self::proxy_group_rows().iter().enumerate() {
            self.ui.label(row.name).apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_primary) }
                },
            );
            self.ui.label(row.meta).apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_muted) }
                },
            );
            self.ui.label(row.status).apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_muted) }
                },
            );
            self.ui.label(row.detail_empty).apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_muted) }
                },
            );
            self.ui.label(row.detail_overflow).apply_over(
                cx,
                live! {
                    draw_text: { color: (palette.text_muted) }
                },
            );
            let item_count = self
                .state
                .proxy_groups
                .get(row_index)
                .map(|group| group.proxies.len().min(Self::MAX_PROXY_OPTIONS_PER_GROUP))
                .unwrap_or(0);
            for item_index in 0..item_count {
                let item = Self::proxy_item_row_ids(row_index, item_index);
                self.ui.view(&item.row).apply_over(
                    cx,
                    live! {
                        draw_bg: { color: (palette.panel_bg) }
                    },
                );
                self.ui.label(&item.name).apply_over(
                    cx,
                    live! {
                        draw_text: { color: (palette.text_primary) }
                    },
                );
                self.ui.label(&item.meta).apply_over(
                    cx,
                    live! {
                        draw_text: { color: (palette.text_muted) }
                    },
                );
                self.ui.label(&item.speed).apply_over(
                    cx,
                    live! {
                        draw_text: { color: (palette.text_muted) }
                    },
                );
            }
        }

        self.ui.label(ids!(dashboard.rules_title)).apply_over(
            cx,
            live! {
                draw_text: { color: (palette.text_primary) }
            },
        );
        self.ui.label(ids!(dashboard.rules_desc)).apply_over(
            cx,
            live! {
                draw_text: { color: (palette.text_muted) }
            },
        );
        self.ui.label(ids!(dashboard.rules_count)).apply_over(
            cx,
            live! {
                draw_text: { color: (palette.text_primary) }
            },
        );
        self.ui.label(ids!(dashboard.rules_empty)).apply_over(
            cx,
            live! {
                draw_text: { color: (palette.text_muted) }
            },
        );
        self.ui.label(ids!(dashboard.rules_list)).apply_over(
            cx,
            live! {
                draw_text: { color: (palette.text_muted) }
            },
        );
    }

    fn apply_menu_button_style(
        &mut self,
        cx: &mut Cx,
        id: &[LiveId; 2],
        active: bool,
        palette: ThemePalette,
    ) {
        let button = self.ui.widget(id);
        if active {
            button.apply_over(
                cx,
                live! {
                    draw_bg: {
                        color: (palette.menu_active_bg),
                        color_hover: (palette.menu_active_hover),
                        color_pressed: (palette.menu_active_pressed)
                    }
                    draw_text: { color: (palette.text_primary) }
                },
            );
        } else {
            button.apply_over(
                cx,
                live! {
                    draw_bg: {
                        color: (palette.menu_inactive_bg),
                        color_hover: (palette.menu_inactive_hover),
                        color_pressed: (palette.menu_inactive_pressed)
                    }
                    draw_text: { color: (palette.text_primary) }
                },
            );
        }
    }

    fn apply_proxy_mode_button_style(
        &mut self,
        cx: &mut Cx,
        id: &[LiveId; 2],
        active: bool,
        palette: ThemePalette,
    ) {
        let button = self.ui.widget(id);
        if active {
            button.apply_over(
                cx,
                live! {
                    draw_bg: {
                        color: (palette.menu_active_bg),
                        color_hover: (palette.menu_active_hover),
                        color_pressed: (palette.menu_active_pressed)
                    }
                    draw_text: { color: (palette.text_primary) }
                },
            );
        } else {
            button.apply_over(
                cx,
                live! {
                    draw_bg: {
                        color: (palette.panel_bg),
                        color_hover: (palette.panel_alt_bg),
                        color_pressed: (palette.panel_alt_bg)
                    }
                    draw_text: { color: (palette.text_muted) }
                },
            );
        }
    }

    fn apply_proxy_mode_buttons(&mut self, cx: &mut Cx) {
        let palette = self.theme_palette();
        self.apply_proxy_mode_button_style(
            cx,
            ids!(dashboard.proxy_mode_rule_btn),
            self.state.proxy_mode == ProxyMode::Rule,
            palette,
        );
        self.apply_proxy_mode_button_style(
            cx,
            ids!(dashboard.proxy_mode_global_btn),
            self.state.proxy_mode == ProxyMode::Global,
            palette,
        );
        self.apply_proxy_mode_button_style(
            cx,
            ids!(dashboard.proxy_mode_direct_btn),
            self.state.proxy_mode == ProxyMode::Direct,
            palette,
        );
    }

    fn update_menu_buttons(&mut self, cx: &mut Cx) {
        let palette = self.theme_palette();
        self.apply_menu_button_style(
            cx,
            ids!(sidebar.menu_profiles),
            self.state.active_page == Page::Profiles,
            palette,
        );
        self.apply_menu_button_style(
            cx,
            ids!(sidebar.menu_proxy_groups),
            self.state.active_page == Page::ProxyGroups,
            palette,
        );
        self.apply_menu_button_style(
            cx,
            ids!(sidebar.menu_rules),
            self.state.active_page == Page::Rules,
            palette,
        );
        self.apply_menu_button_style(
            cx,
            ids!(sidebar.menu_settings),
            self.state.active_page == Page::Settings,
            palette,
        );
    }

    fn switch_page(&mut self, cx: &mut Cx, page: Page) {
        if self.state.active_page == page {
            return;
        }
        self.state.active_page = page;
        self.ui
            .view(ids!(dashboard.content_body))
            .set_scroll_pos(cx, dvec2(0.0, 0.0));
        self.refresh_ui(cx);
    }
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        Self::init_logging();
        self.silent_start_requested = Self::startup_silent_start_requested();
        self.silent_start_applied = false;
        info!("linkpad startup begin");
        self.load_persisted_settings();
        let _ = self.core.configure_startup(
            self.state.auto_launch_enabled,
            self.state.silent_start_enabled,
        );
        self.sync_startup_state_from_core();
        self.apply_clash_config_to_core();
        self.load_persisted_profiles();
        self.warmup_core_runtime_on_startup();
        self.sync_from_core();
        self.persist_profiles();
        self.set_import_status_ready();
        self.install_shell_integrations();
        self.apply_silent_start_visibility(cx);
        info!(
            "linkpad startup complete: profiles={}, groups={}, rules={}",
            self.state.profiles.len(),
            self.state.proxy_groups.len(),
            self.state.rules.len()
        );
        self.refresh_ui(cx);
    }

    fn handle_timer(&mut self, cx: &mut Cx, event: &TimerEvent) {
        if self.import_poll_timer.is_timer(event).is_some() {
            self.poll_profile_import(cx);
        }
        if self.latency_poll_timer.is_timer(event).is_some() {
            self.poll_latency_test(cx);
        }
        if self.locate_timer.is_timer(event).is_some() {
            self.perform_pending_locate(cx);
        }
        if self.notification_timer.is_timer(event).is_some() {
            self.dismiss_notification(cx);
            self.refresh_ui(cx);
        }
        if self.core_task_timer.is_timer(event).is_some() {
            self.poll_core_task(cx);
        }
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        for action in actions {
            if let Some(shell_action) = action.downcast_ref::<tray::ShellCommandAction>() {
                self.apply_shell_command(cx, shell_action.0);
            }
            if action.downcast_ref::<tray::TrayActivateAction>().is_some() {
                self.handle_tray_activate(cx);
            }
        }

        if self
            .ui
            .mp_button(ids!(sidebar.menu_profiles))
            .clicked(actions)
        {
            self.switch_page(cx, Page::Profiles);
        }
        if self
            .ui
            .mp_button(ids!(sidebar.menu_proxy_groups))
            .clicked(actions)
        {
            self.switch_page(cx, Page::ProxyGroups);
        }
        if self.ui.mp_button(ids!(sidebar.menu_rules)).clicked(actions) {
            self.switch_page(cx, Page::Rules);
        }
        if self
            .ui
            .mp_button(ids!(sidebar.menu_settings))
            .clicked(actions)
        {
            self.switch_page(cx, Page::Settings);
        }

        self.handle_profiles_actions(cx, actions);
        self.handle_proxy_groups_actions(cx, actions);
        self.handle_rules_actions(cx, actions);
        self.handle_settings_actions(cx, actions);
    }
}
