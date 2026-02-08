use super::*;

impl App {
    pub(super) fn handle_proxy_groups_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        for (index, row_ids) in Self::proxy_group_rows().iter().enumerate() {
            if self.ui.mp_button(row_ids.open_btn).clicked(actions) {
                self.select_active_proxy_group(cx, index);
            }
            if self.ui.mp_button(row_ids.test_btn).clicked(actions) {
                self.start_latency_test_for_group(cx, index);
            }
            if self.ui.mp_button(row_ids.locate_btn).clicked(actions) {
                self.locate_selected_proxy_for_group(cx, index);
            }
        }
        if self
            .ui
            .mp_button(ids!(dashboard.proxy_mode_rule_btn))
            .clicked(actions)
        {
            self.set_proxy_mode(cx, ProxyMode::Rule);
        }
        if self
            .ui
            .mp_button(ids!(dashboard.proxy_mode_global_btn))
            .clicked(actions)
        {
            self.set_proxy_mode(cx, ProxyMode::Global);
        }
        if self
            .ui
            .mp_button(ids!(dashboard.proxy_mode_direct_btn))
            .clicked(actions)
        {
            self.set_proxy_mode(cx, ProxyMode::Direct);
        }
        for (group_index, _) in Self::proxy_group_rows().iter().enumerate() {
            let item_count = self
                .state
                .proxy_groups
                .get(group_index)
                .map(|group| group.proxies.len().min(Self::MAX_PROXY_OPTIONS_PER_GROUP))
                .unwrap_or(0);
            for item_index in 0..item_count {
                let row_ids = Self::proxy_item_row_ids(group_index, item_index);
                if self.ui.mp_button(&row_ids.select_btn).clicked(actions) {
                    self.select_proxy_item_for_group(cx, group_index, item_index);
                }
            }
        }
    }

    pub(super) fn apply_saved_proxy_group_selections(&mut self) {
        for group in &self.state.proxy_groups {
            let Some(saved_proxy_name) = self.saved_proxy_group_selections.get(&group.name) else {
                continue;
            };
            let Some(index) = group
                .proxies
                .iter()
                .position(|name| name == saved_proxy_name)
            else {
                continue;
            };
            self.state
                .proxy_group_selected
                .insert(group.name.clone(), index);
        }
    }

    pub(super) fn snapshot_proxy_group_selections(&mut self) {
        for group in &self.state.proxy_groups {
            let Some(selected_index) = self.state.proxy_group_selected.get(&group.name).copied()
            else {
                continue;
            };
            let Some(selected_proxy_name) = group.proxies.get(selected_index) else {
                continue;
            };
            self.saved_proxy_group_selections
                .insert(group.name.clone(), selected_proxy_name.clone());
        }
    }

    pub(super) fn apply_saved_proxy_group_selections_to_core(&mut self) {
        if self.state.proxy_groups.is_empty() {
            return;
        }
        for group in &self.state.proxy_groups {
            let Some(selected_index) = self.state.proxy_group_selected.get(&group.name).copied()
            else {
                continue;
            };
            let Some(selected_proxy_name) = group.proxies.get(selected_index) else {
                continue;
            };
            let _ = self.core.select_proxy(&group.name, selected_proxy_name);
        }
    }

    fn select_active_proxy_group(&mut self, cx: &mut Cx, row_index: usize) {
        let Some(group_name) = self
            .state
            .proxy_groups
            .get(row_index)
            .map(|group| group.name.clone())
        else {
            return;
        };
        if self.state.active_proxy_group.as_deref() == Some(group_name.as_str()) {
            self.state.active_proxy_group = None;
        } else {
            self.state.active_proxy_group = Some(group_name);
        }
        self.refresh_ui(cx);
    }

    fn select_proxy_item_for_group(&mut self, cx: &mut Cx, row_index: usize, item_index: usize) {
        let Some((group_name, proxy_name, proxy_len)) =
            self.state.proxy_groups.get(row_index).and_then(|group| {
                group
                    .proxies
                    .get(item_index)
                    .cloned()
                    .map(|proxy_name| (group.name.clone(), proxy_name, group.proxies.len()))
            })
        else {
            return;
        };
        if item_index >= proxy_len {
            return;
        }
        info!("proxy selection requested: group={group_name}, index={item_index}");
        match self.core.select_proxy(&group_name, &proxy_name) {
            Ok(()) => {
                self.state
                    .proxy_group_selected
                    .insert(group_name, item_index);
                self.snapshot_proxy_group_selections();
                self.persist_settings();
                info!("proxy selection applied: proxy={proxy_name}");
            }
            Err(error) => {
                let strings = i18n::strings(self.state.language);
                error!("proxy selection failed: {error}");
                self.push_notification(
                    cx,
                    NotificationLevel::Error,
                    format!("{}: {error}", strings.proxy_groups_select_failed_prefix),
                );
            }
        }
        self.refresh_ui(cx);
    }

    fn start_latency_test_for_group(&mut self, cx: &mut Cx, row_index: usize) {
        if self.latency_rx.is_some() {
            return;
        }

        let Some(group) = self.state.proxy_groups.get(row_index).cloned() else {
            return;
        };
        if group.proxies.is_empty() {
            let strings = i18n::strings(self.state.language);
            self.push_notification(
                cx,
                NotificationLevel::Info,
                strings.proxy_groups_proxy_empty.to_string(),
            );
            return;
        }

        for proxy_name in &group.proxies {
            self.proxy_latency_ms
                .insert(proxy_name.clone(), LatencyStatus::NotTested);
        }

        let core = self.core.clone();
        let group_name_for_ui = group.name.clone();
        let group_name_for_task = group.name;
        let proxies = group.proxies;
        let (tx, rx) = std::sync::mpsc::channel::<LatencyTaskEvent>();
        thread::spawn(move || {
            let mut success_count = 0usize;
            let mut first_error: Option<String> = None;
            let total_count = proxies.len();

            for proxy_name in proxies {
                let delay = match core.probe_proxy_delay(&proxy_name) {
                    Ok(delay) => {
                        if delay.is_some() {
                            success_count += 1;
                        }
                        delay
                    }
                    Err(error) => {
                        if first_error.is_none() {
                            first_error = Some(error.to_string());
                        }
                        None
                    }
                };

                let _ = tx.send(LatencyTaskEvent::Progress {
                    group_name: group_name_for_task.clone(),
                    proxy_name,
                    delay,
                });
            }

            if success_count == 0 {
                if let Some(error) = first_error {
                    let _ = tx.send(LatencyTaskEvent::Failed {
                        group_name: group_name_for_task,
                        error,
                    });
                    return;
                }
            }

            let _ = tx.send(LatencyTaskEvent::Finished {
                group_name: group_name_for_task,
                success_count,
                total_count,
            });
        });

        if !self.latency_poll_timer.is_empty() {
            cx.stop_timer(self.latency_poll_timer);
        }
        self.latency_rx = Some(rx);
        self.latency_testing_group = Some(group_name_for_ui);
        self.latency_poll_timer = cx.start_interval(0.1);
        self.refresh_ui(cx);
    }

    fn stop_latency_polling(&mut self, cx: &mut Cx) {
        if !self.latency_poll_timer.is_empty() {
            cx.stop_timer(self.latency_poll_timer);
            self.latency_poll_timer = Timer::default();
        }
        self.latency_rx = None;
        self.latency_testing_group = None;
    }

    pub(super) fn poll_latency_test(&mut self, cx: &mut Cx) {
        let mut events = Vec::new();
        let disconnected = {
            let Some(latency_rx) = self.latency_rx.as_ref() else {
                return;
            };

            loop {
                match latency_rx.try_recv() {
                    Ok(event) => events.push(event),
                    Err(TryRecvError::Empty) => break false,
                    Err(TryRecvError::Disconnected) => break true,
                }
            }
        };

        if events.is_empty() && !disconnected {
            return;
        }

        let strings = i18n::strings(self.state.language);
        let mut should_stop = false;

        for event in events {
            match event {
                LatencyTaskEvent::Progress {
                    group_name,
                    proxy_name,
                    delay,
                } => {
                    if self
                        .latency_testing_group
                        .as_deref()
                        .map(|active| active == group_name.as_str())
                        .unwrap_or(false)
                    {
                        let status = delay
                            .map(LatencyStatus::Value)
                            .unwrap_or(LatencyStatus::Timeout);
                        self.proxy_latency_ms.insert(proxy_name, status);
                    }
                }
                LatencyTaskEvent::Finished {
                    group_name,
                    success_count,
                    total_count,
                } => {
                    should_stop = true;
                    self.push_notification(
                        cx,
                        NotificationLevel::Success,
                        format!(
                            "{}: {} ({}/{})",
                            strings.proxy_groups_latency_test_success_prefix,
                            group_name,
                            success_count,
                            total_count
                        ),
                    );
                }
                LatencyTaskEvent::Failed { group_name, error } => {
                    should_stop = true;
                    self.push_notification(
                        cx,
                        NotificationLevel::Error,
                        format!(
                            "{}: {} ({error})",
                            strings.proxy_groups_latency_test_failed_prefix, group_name
                        ),
                    );
                }
            }
        }

        if disconnected && !should_stop {
            should_stop = true;
            self.push_notification(
                cx,
                NotificationLevel::Error,
                format!(
                    "{}: latency worker disconnected",
                    strings.proxy_groups_latency_test_failed_prefix
                ),
            );
        }

        if should_stop {
            self.stop_latency_polling(cx);
        }

        self.refresh_ui(cx);
    }

    fn locate_selected_proxy_for_group(&mut self, cx: &mut Cx, row_index: usize) {
        let Some(group) = self.state.proxy_groups.get(row_index).cloned() else {
            return;
        };
        let Some(proxy_count) = (!group.proxies.is_empty()).then_some(group.proxies.len()) else {
            let strings = i18n::strings(self.state.language);
            self.push_notification(
                cx,
                NotificationLevel::Info,
                strings.proxy_groups_locate_failed.to_string(),
            );
            return;
        };

        let mut selected_index = self
            .state
            .proxy_group_selected
            .get(&group.name)
            .copied()
            .unwrap_or(0);
        if selected_index >= proxy_count {
            selected_index = 0;
        }

        self.state.active_proxy_group = Some(group.name);
        self.pending_locate = Some((row_index, selected_index));
        self.locate_retry_count = 0;
        if !self.locate_timer.is_empty() {
            cx.stop_timer(self.locate_timer);
        }
        self.locate_timer = cx.start_timeout(0.04);
        self.refresh_ui(cx);
    }

    pub(super) fn perform_pending_locate(&mut self, cx: &mut Cx) {
        let Some((row_index, item_index)) = self.pending_locate else {
            return;
        };

        let item = Self::proxy_item_row_ids(row_index, item_index);
        let content_area = self.ui.view(ids!(dashboard.content_body)).area();
        let target_area = self.ui.view(&item.row).area();

        if !content_area.is_empty() && !target_area.is_empty() {
            let content_rect = content_area.rect(cx);
            let target_rect = target_area.rect(cx);
            let target_y = (target_rect.pos.y - content_rect.pos.y - 24.0).max(0.0);
            self.ui
                .view(ids!(dashboard.content_body))
                .set_scroll_pos(cx, dvec2(0.0, target_y));
            self.pending_locate = None;
            self.locate_retry_count = 0;
            if !self.locate_timer.is_empty() {
                cx.stop_timer(self.locate_timer);
            }
            self.locate_timer = Timer::default();
            self.refresh_ui(cx);
            return;
        }

        if self.locate_retry_count < 6 {
            self.locate_retry_count += 1;
            if !self.locate_timer.is_empty() {
                cx.stop_timer(self.locate_timer);
            }
            self.locate_timer = cx.start_timeout(0.05);
            return;
        }

        let fallback_y = Self::estimate_proxy_item_scroll_y(row_index, item_index);
        self.ui
            .view(ids!(dashboard.content_body))
            .set_scroll_pos(cx, dvec2(0.0, fallback_y));
        self.pending_locate = None;
        self.locate_retry_count = 0;
        if !self.locate_timer.is_empty() {
            cx.stop_timer(self.locate_timer);
        }
        self.locate_timer = Timer::default();
        self.refresh_ui(cx);
    }

    fn estimate_proxy_item_scroll_y(row_index: usize, item_index: usize) -> f64 {
        let card_top_padding = 220.0;
        let collapsed_group_height = 90.0;
        let group_header_height = 72.0;
        let option_row_height = 96.0;
        let option_row_index = item_index / 2;
        card_top_padding
            + (row_index as f64 * collapsed_group_height)
            + group_header_height
            + (option_row_index as f64 * option_row_height)
    }
}
