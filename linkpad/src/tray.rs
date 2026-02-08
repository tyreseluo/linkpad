use super::*;
use makepad_components::makepad_widgets::makepad_platform::CxOsOp;
use makepad_components::shell::{
    CommandId, Tray, TrayCommandItem, TrayIcon, TrayMenuItem, TrayMenuItemRole, TrayMenuModel,
    TrayModel,
};

const TRAY_CMD_MODE_RULE: u64 = 10_001;
const TRAY_CMD_MODE_GLOBAL: u64 = 10_002;
const TRAY_CMD_MODE_DIRECT: u64 = 10_003;
const TRAY_CMD_PROFILE_BASE: u64 = 20_000;
const TRAY_CMD_SYSTEM_PROXY_TOGGLE: u64 = 30_001;
const TRAY_CMD_EXIT: u64 = 30_002;

#[derive(Clone, Debug)]
pub(super) struct ShellCommandAction(pub CommandId);

#[derive(Clone, Debug)]
pub(super) struct TrayActivateAction;

impl App {
    pub(super) fn install_shell_integrations(&mut self) {
        self.install_app_menu_placeholder();
        self.install_tray();
    }

    fn install_app_menu_placeholder(&mut self) {
        #[cfg(target_os = "macos")]
        {
            if self.app_menu_installed {
                return;
            }
            // Reserved for future native app-menu integration.
            self.app_menu_installed = true;
        }
    }

    fn install_tray(&mut self) {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            if self.tray.is_some() {
                return;
            }

            let strings = i18n::strings(self.state.language);
            let icon_bytes: &[u8] = include_bytes!("../assets/tray.png");
            let icon = TrayIcon::from_png_bytes(icon_bytes.to_vec()).with_template(true);
            let model =
                TrayModel::new(icon, self.build_tray_menu(strings)).with_tooltip(strings.app_name);

            let result = Tray::create(
                model,
                |cmd| {
                    Cx::post_action(ShellCommandAction(cmd));
                },
                || {
                    Cx::post_action(TrayActivateAction);
                },
            );

            match result {
                Ok(handle) => {
                    self.tray = Some(handle);
                }
                Err(error) => {
                    warn!("failed to create tray: {:?}", error);
                }
            }
        }
    }

    pub(super) fn handle_tray_activate(&mut self, cx: &mut Cx) {
        let Some(window_id) = self.window_id else {
            return;
        };
        cx.push_unique_platform_op(CxOsOp::RestoreWindow(window_id));
    }

    pub(super) fn sync_tray_menu(&mut self, strings: &i18n::Strings) {
        let menu = self.build_tray_menu(strings);
        if let Some(tray) = self.tray.as_mut() {
            let _ = tray.update_menu(menu);
            let _ = tray.update_tooltip(Some(strings.app_name.to_string()));
        }
    }

    fn build_tray_menu(&self, strings: &i18n::Strings) -> TrayMenuModel {
        let outbound_title = format!(
            "{} ({})",
            strings.tray_outbound_modes,
            Self::tray_proxy_mode_label(strings, self.state.proxy_mode)
        );

        let mut mode_rule_item = TrayCommandItem::new(
            CommandId::new(TRAY_CMD_MODE_RULE).expect("valid tray command id"),
            strings.proxy_mode_rule,
        );
        mode_rule_item.checked = self.state.proxy_mode == ProxyMode::Rule;

        let mut mode_global_item = TrayCommandItem::new(
            CommandId::new(TRAY_CMD_MODE_GLOBAL).expect("valid tray command id"),
            strings.proxy_mode_global,
        );
        mode_global_item.checked = self.state.proxy_mode == ProxyMode::Global;

        let mut mode_direct_item = TrayCommandItem::new(
            CommandId::new(TRAY_CMD_MODE_DIRECT).expect("valid tray command id"),
            strings.proxy_mode_direct,
        );
        mode_direct_item.checked = self.state.proxy_mode == ProxyMode::Direct;

        let profile_items = if self.state.profiles.is_empty() {
            let mut empty_item = TrayCommandItem::new(
                CommandId::new(TRAY_CMD_PROFILE_BASE).expect("valid tray command id"),
                strings.profiles_list_empty,
            );
            empty_item.enabled = false;
            vec![TrayMenuItem::Command(empty_item)]
        } else {
            self.state
                .profiles
                .iter()
                .enumerate()
                .filter_map(|(index, profile)| {
                    let command_id = TRAY_CMD_PROFILE_BASE + index as u64;
                    CommandId::new(command_id).map(|id| {
                        let mut item = TrayCommandItem::new(id, &profile.name);
                        item.checked = profile.active;
                        TrayMenuItem::Command(item)
                    })
                })
                .collect::<Vec<_>>()
        };

        let mut system_proxy_item = TrayCommandItem::new(
            CommandId::new(TRAY_CMD_SYSTEM_PROXY_TOGGLE).expect("valid tray command id"),
            strings.tray_system_proxy,
        );
        system_proxy_item.checked = self.state.system_proxy_enabled;

        TrayMenuModel::new(vec![
            TrayMenuItem::Submenu(makepad_components::shell::TraySubmenu::new(
                outbound_title,
                vec![
                    TrayMenuItem::Command(mode_rule_item),
                    TrayMenuItem::Command(mode_direct_item),
                    TrayMenuItem::Command(mode_global_item),
                ],
            )),
            TrayMenuItem::Submenu(makepad_components::shell::TraySubmenu::new(
                strings.tray_profiles,
                profile_items,
            )),
            TrayMenuItem::Separator,
            TrayMenuItem::Command(system_proxy_item),
            TrayMenuItem::Separator,
            TrayMenuItem::Command(
                TrayCommandItem::new(
                    CommandId::new(TRAY_CMD_EXIT).expect("valid tray command id"),
                    strings.tray_exit,
                )
                .with_role(TrayMenuItemRole::Quit),
            ),
        ])
    }

    fn tray_proxy_mode_label<'a>(strings: &'a i18n::Strings, mode: ProxyMode) -> &'a str {
        match mode {
            ProxyMode::Rule => strings.proxy_mode_rule,
            ProxyMode::Global => strings.proxy_mode_global,
            ProxyMode::Direct => strings.proxy_mode_direct,
        }
    }

    pub(super) fn apply_shell_command(&mut self, cx: &mut Cx, command_id: CommandId) {
        let raw_id = command_id.as_u64();
        match raw_id {
            TRAY_CMD_MODE_RULE => self.set_proxy_mode(cx, ProxyMode::Rule),
            TRAY_CMD_MODE_GLOBAL => self.set_proxy_mode(cx, ProxyMode::Global),
            TRAY_CMD_MODE_DIRECT => self.set_proxy_mode(cx, ProxyMode::Direct),
            TRAY_CMD_SYSTEM_PROXY_TOGGLE => {
                self.set_system_proxy_enabled(cx, !self.state.system_proxy_enabled);
            }
            TRAY_CMD_EXIT => cx.quit(),
            _ => {
                if raw_id >= TRAY_CMD_PROFILE_BASE {
                    let profile_index = (raw_id - TRAY_CMD_PROFILE_BASE) as usize;
                    if profile_index < self.state.profiles.len() {
                        self.activate_profile_row(cx, profile_index);
                    }
                }
            }
        }
    }
}
