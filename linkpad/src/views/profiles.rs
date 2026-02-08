use super::*;

impl App {
    pub(super) fn handle_profiles_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if let Some(value) = self
            .ui
            .text_input(ids!(dashboard.profile_url_input))
            .changed(actions)
        {
            self.state.profile_url_input = value;
        }
        if self
            .ui
            .mp_button(ids!(dashboard.profile_import_btn))
            .clicked(actions)
        {
            self.import_profile_from_input(cx);
        }
        if self
            .ui
            .mp_button(ids!(dashboard.profile_row_1_activate_btn))
            .clicked(actions)
        {
            self.activate_profile_row(cx, 0);
        }
        if self
            .ui
            .mp_button(ids!(dashboard.profile_row_2_activate_btn))
            .clicked(actions)
        {
            self.activate_profile_row(cx, 1);
        }
        if self
            .ui
            .mp_button(ids!(dashboard.profile_row_3_activate_btn))
            .clicked(actions)
        {
            self.activate_profile_row(cx, 2);
        }
        if self
            .ui
            .mp_button(ids!(dashboard.profile_row_1_refresh_btn))
            .clicked(actions)
        {
            self.refresh_profile_row(cx, 0);
        }
        if self
            .ui
            .mp_button(ids!(dashboard.profile_row_2_refresh_btn))
            .clicked(actions)
        {
            self.refresh_profile_row(cx, 1);
        }
        if self
            .ui
            .mp_button(ids!(dashboard.profile_row_3_refresh_btn))
            .clicked(actions)
        {
            self.refresh_profile_row(cx, 2);
        }
        if self
            .ui
            .mp_button(ids!(dashboard.profile_row_1_delete_btn))
            .clicked(actions)
        {
            self.delete_profile_row(cx, 0);
        }
        if self
            .ui
            .mp_button(ids!(dashboard.profile_row_2_delete_btn))
            .clicked(actions)
        {
            self.delete_profile_row(cx, 1);
        }
        if self
            .ui
            .mp_button(ids!(dashboard.profile_row_3_delete_btn))
            .clicked(actions)
        {
            self.delete_profile_row(cx, 2);
        }
    }

    pub(super) fn set_import_status_ready(&mut self) {
        self.state.import_status.message = i18n::strings(self.state.language)
            .profiles_import_ready
            .to_string();
        self.state.import_status.is_error = false;
    }

    fn set_import_status_error(&mut self, message: String) {
        self.state.import_status.message = message;
        self.state.import_status.is_error = true;
    }

    fn stop_import_polling(&mut self, cx: &mut Cx) {
        if !self.import_poll_timer.is_empty() {
            cx.stop_timer(self.import_poll_timer);
            self.import_poll_timer = Timer::default();
        }
        self.import_rx = None;
    }

    fn import_profile_from_input(&mut self, cx: &mut Cx) {
        if self.import_rx.is_some() {
            warn!("skip profile import: previous import task still running");
            return;
        }

        let strings = i18n::strings(self.state.language);
        let url = self.state.profile_url_input.trim().to_string();
        if url.is_empty() {
            warn!("skip profile import: url is empty");
            let message = strings.profiles_import_error.to_string();
            self.set_import_status_error(message.clone());
            self.push_notification(cx, NotificationLevel::Error, message);
            self.refresh_ui(cx);
            return;
        }
        info!("profile import requested");
        if !self.import_poll_timer.is_empty() {
            cx.stop_timer(self.import_poll_timer);
            self.import_poll_timer = Timer::default();
        }

        self.state.import_status.message = strings.profiles_import_loading.to_string();
        self.state.import_status.is_error = false;

        let core = self.core.clone();
        let (tx, rx) = std::sync::mpsc::channel::<ImportTaskResult>();
        thread::spawn(move || {
            let result = core
                .import_profile_url(&url, true)
                .map(|_| ())
                .map_err(|error| error.to_string());
            let _ = tx.send(result);
        });

        self.import_rx = Some(rx);
        self.import_poll_timer = cx.start_interval(0.1);
        self.refresh_ui(cx);
    }

    pub(super) fn poll_profile_import(&mut self, cx: &mut Cx) {
        let Some(import_rx) = self.import_rx.as_ref() else {
            return;
        };

        let result = match import_rx.try_recv() {
            Ok(result) => Some(result),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(Err("import worker disconnected".to_string())),
        };

        let Some(result) = result else {
            return;
        };

        self.stop_import_polling(cx);
        match result {
            Ok(()) => {
                let strings = i18n::strings(self.state.language);
                self.persist_profiles();
                self.sync_from_core();
                self.state.profile_url_input.clear();
                self.state.import_status.message = strings.profiles_import_success.to_string();
                self.state.import_status.is_error = false;
                info!("profile import succeeded");
                self.push_notification(
                    cx,
                    NotificationLevel::Success,
                    strings.profiles_import_success.to_string(),
                );
            }
            Err(error) => {
                let strings = i18n::strings(self.state.language);
                let message = format!("{} ({error})", strings.profiles_import_error);
                self.set_import_status_error(message.clone());
                error!("profile import failed: {error}");
                self.push_notification(cx, NotificationLevel::Error, message);
            }
        }
        self.refresh_ui(cx);
    }

    pub(super) fn activate_profile_row(&mut self, cx: &mut Cx, row_index: usize) {
        let Some(profile_id) = self
            .state
            .profiles
            .get(row_index)
            .map(|profile| profile.id.clone())
        else {
            return;
        };

        match self.core.set_active_profile(&profile_id) {
            Ok(()) => {
                self.persist_profiles();
                self.sync_from_core();
                self.set_import_status_ready();
            }
            Err(error) => {
                self.set_import_status_error(format!("{error}"));
            }
        }
        self.refresh_ui(cx);
    }

    fn refresh_profile_row(&mut self, cx: &mut Cx, row_index: usize) {
        let Some(profile_id) = self
            .state
            .profiles
            .get(row_index)
            .map(|profile| profile.id.clone())
        else {
            return;
        };

        match self.core.refresh_profile(&profile_id) {
            Ok(_) => {
                self.persist_profiles();
                self.sync_from_core();
                self.set_import_status_ready();
            }
            Err(error) => {
                self.set_import_status_error(format!("{error}"));
            }
        }
        self.refresh_ui(cx);
    }

    fn delete_profile_row(&mut self, cx: &mut Cx, row_index: usize) {
        let Some(profile_id) = self
            .state
            .profiles
            .get(row_index)
            .map(|profile| profile.id.clone())
        else {
            return;
        };

        match self.core.delete_profile(&profile_id) {
            Ok(()) => {
                self.persist_profiles();
                self.sync_from_core();
                self.set_import_status_ready();
            }
            Err(error) => {
                self.set_import_status_error(format!("{error}"));
            }
        }
        self.refresh_ui(cx);
    }

    pub(super) fn load_persisted_profiles(&mut self) {
        let profiles = profile_store::load();
        if profiles.is_empty() {
            info!("no persisted profiles found");
            return;
        }
        info!("loaded persisted profiles: count={}", profiles.len());
        self.core.replace_profiles(profiles);
    }

    pub(super) fn persist_profiles(&self) {
        let profiles = self.core.profiles();
        let _ = profile_store::save(&profiles);
    }
}
