use super::*;

impl App {
    pub(super) fn handle_settings_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if let Some(value) = self
            .ui
            .text_input(ids!(dashboard.clash_port_input))
            .changed(actions)
        {
            self.state.clash_port_input = value;
        }
        if let Some(on) = self
            .ui
            .mp_switch(ids!(dashboard.system_proxy_switch))
            .changed(actions)
        {
            self.set_system_proxy_enabled(cx, on);
        }
        if let Some(on) = self
            .ui
            .mp_switch(ids!(dashboard.close_to_tray_switch))
            .changed(actions)
        {
            self.state.close_to_tray_enabled = on;
            self.persist_settings();
            self.refresh_ui(cx);
        }
        if let Some(on) = self
            .ui
            .mp_switch(ids!(dashboard.auto_launch_switch))
            .changed(actions)
        {
            let previous = self.state.auto_launch_enabled;
            self.state.auto_launch_enabled = on;
            if let Err(error) = self.core.configure_startup(
                self.state.auto_launch_enabled,
                self.state.silent_start_enabled,
            ) {
                let strings = i18n::strings(self.state.language);
                self.state.auto_launch_enabled = previous;
                self.push_notification(
                    cx,
                    NotificationLevel::Error,
                    format!("{}: {error}", strings.auto_launch_update_failed_prefix),
                );
            } else {
                self.sync_startup_state_from_core();
                self.persist_settings();
            }
            self.refresh_ui(cx);
        }
        if let Some(on) = self
            .ui
            .mp_switch(ids!(dashboard.silent_start_switch))
            .changed(actions)
        {
            let previous = self.state.silent_start_enabled;
            self.state.silent_start_enabled = on;
            if self.state.auto_launch_enabled {
                if let Err(error) = self.core.configure_startup(
                    self.state.auto_launch_enabled,
                    self.state.silent_start_enabled,
                ) {
                    let strings = i18n::strings(self.state.language);
                    self.state.silent_start_enabled = previous;
                    self.push_notification(
                        cx,
                        NotificationLevel::Error,
                        format!("{}: {error}", strings.silent_start_update_failed_prefix),
                    );
                } else {
                    self.sync_startup_state_from_core();
                }
            }
            self.persist_settings();
            self.refresh_ui(cx);
        }
        if let Some(index) = self
            .ui
            .drop_down(ids!(dashboard.language_dropdown))
            .changed(actions)
        {
            self.state.language = Language::from_index(index);
            self.persist_settings();
            self.refresh_ui(cx);
        }
        if let Some(index) = self
            .ui
            .drop_down(ids!(dashboard.theme_dropdown))
            .changed(actions)
        {
            self.state.theme = ThemePreference::from_index(index);
            self.persist_settings();
            self.refresh_ui(cx);
        }
        if self
            .ui
            .mp_button(ids!(dashboard.clash_core_upgrade_btn))
            .clicked(actions)
        {
            self.start_core_upgrade(cx);
        }
        if self
            .ui
            .mp_button(ids!(dashboard.clash_core_restart_btn))
            .clicked(actions)
        {
            self.start_core_restart(cx);
        }
        if self
            .ui
            .mp_button(ids!(dashboard.clash_port_save_btn))
            .clicked(actions)
        {
            let strings = i18n::strings(self.state.language);
            let trimmed = self.state.clash_port_input.trim();
            let parsed = trimmed.parse::<u16>().ok().filter(|port| *port > 0);
            if let Some(port) = parsed {
                let mut config = self.core.config();
                config.mixed_port = port;
                match self.core.update_config(config) {
                    Ok(()) => {
                        self.state.clash_mixed_port = port;
                        self.state.clash_port_input = port.to_string();
                        self.persist_settings();
                        self.push_notification(
                            cx,
                            NotificationLevel::Success,
                            strings.clash_port_update_success.to_string(),
                        );
                    }
                    Err(error) => {
                        self.push_notification(
                            cx,
                            NotificationLevel::Error,
                            format!("{}: {error}", strings.clash_port_update_failed_prefix),
                        );
                    }
                }
            } else {
                self.push_notification(
                    cx,
                    NotificationLevel::Error,
                    strings.clash_port_update_invalid.to_string(),
                );
            }
            self.refresh_ui(cx);
        }
    }

    pub(super) fn set_system_proxy_enabled(&mut self, cx: &mut Cx, on: bool) {
        let strings = i18n::strings(self.state.language);
        info!("system proxy toggle requested: on={on}");
        let result = if on {
            self.core.enable_system_proxy()
        } else {
            self.core.disable_system_proxy()
        };

        match result {
            Ok(()) => {
                self.state.system_proxy_enabled = on;
                if on {
                    self.apply_saved_proxy_group_selections_to_core();
                    self.sync_from_core();
                    self.snapshot_proxy_group_selections();
                }
                let message = if on {
                    strings.system_proxy_enable_success
                } else {
                    strings.system_proxy_disable_success
                };
                info!("system proxy toggle succeeded: on={on}");
                self.push_notification(cx, NotificationLevel::Success, message.to_string());
            }
            Err(error) => {
                self.state.system_proxy_enabled = self.core.is_system_proxy_enabled();
                let message = if on {
                    format!("{}: {error}", strings.system_proxy_enable_failed_prefix)
                } else {
                    format!("{}: {error}", strings.system_proxy_disable_failed_prefix)
                };
                error!("system proxy toggle failed: {error}");
                self.push_notification(cx, NotificationLevel::Error, message);
            }
        }
        self.persist_settings();
        self.refresh_ui(cx);
    }

    pub(super) fn load_persisted_settings(&mut self) {
        if let Some(loaded) = settings_store::load() {
            self.state.language = loaded.language;
            self.state.theme = loaded.theme;
            self.state.system_proxy_enabled = loaded.system_proxy_enabled;
            self.state.close_to_tray_enabled = loaded.close_to_tray_enabled;
            self.state.auto_launch_enabled = loaded.auto_launch_enabled;
            self.state.silent_start_enabled = loaded.silent_start_enabled;
            self.state.clash_mixed_port = loaded.clash_mixed_port;
            self.state.clash_port_input = loaded.clash_mixed_port.to_string();
            self.saved_proxy_group_selections = loaded.proxy_group_selections;
            info!("loaded persisted settings");
        } else {
            info!("no persisted settings found, using defaults");
        }
    }

    pub(super) fn sync_startup_state_from_core(&mut self) {
        if let Ok(status) = self.core.startup_status() {
            self.state.auto_launch_enabled = status.auto_launch;
            if status.auto_launch {
                self.state.silent_start_enabled = status.silent_start;
            }
        }
    }

    pub(super) fn persist_settings(&self) {
        let _ = settings_store::save(
            self.state.language,
            self.state.theme,
            self.state.system_proxy_enabled,
            self.state.close_to_tray_enabled,
            self.state.auto_launch_enabled,
            self.state.silent_start_enabled,
            self.state.clash_mixed_port,
            &self.saved_proxy_group_selections,
        );
    }

    pub(super) fn apply_clash_config_to_core(&mut self) {
        let mut config = self.core.config();
        config.mixed_port = self.state.clash_mixed_port;
        let _ = self.core.update_config(config);
    }
}
