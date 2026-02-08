use super::*;

impl App {
    pub(super) fn handle_rules_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if let Some(value) = self
            .ui
            .text_input(ids!(dashboard.rules_search_input))
            .changed(actions)
        {
            self.state.rules_query = value;
            self.reset_rules_pagination();
            self.refresh_ui(cx);
        }
        if self
            .ui
            .mp_button(ids!(dashboard.rules_filter_all_btn))
            .clicked(actions)
        {
            self.state.rules_filter = RuleFilter::All;
            self.reset_rules_pagination();
            self.refresh_ui(cx);
        }
        if self
            .ui
            .mp_button(ids!(dashboard.rules_filter_domain_btn))
            .clicked(actions)
        {
            self.state.rules_filter = RuleFilter::Domain;
            self.reset_rules_pagination();
            self.refresh_ui(cx);
        }
        if self
            .ui
            .mp_button(ids!(dashboard.rules_filter_ip_cidr_btn))
            .clicked(actions)
        {
            self.state.rules_filter = RuleFilter::IpCidr;
            self.reset_rules_pagination();
            self.refresh_ui(cx);
        }
        if self
            .ui
            .mp_button(ids!(dashboard.rules_filter_process_btn))
            .clicked(actions)
        {
            self.state.rules_filter = RuleFilter::ProcessName;
            self.reset_rules_pagination();
            self.refresh_ui(cx);
        }
    }

    pub(super) fn filtered_rules(&self) -> Vec<String> {
        let query = self.state.rules_query.trim().to_ascii_lowercase();
        self.state
            .rules
            .iter()
            .filter(|rule| Self::rule_matches_filter(rule, self.state.rules_filter))
            .filter(|rule| query.is_empty() || rule.to_ascii_lowercase().contains(&query))
            .cloned()
            .collect()
    }

    pub(super) fn should_paginate_rules(&self) -> bool {
        self.state.rules_filter == RuleFilter::All && self.state.rules_query.trim().is_empty()
    }

    pub(super) fn reset_rules_pagination(&mut self) {
        self.state.rules_visible_count = Self::RULES_PAGE_SIZE;
    }

    pub(super) fn ensure_rules_visible_count(&mut self, filtered_count: usize) {
        if !self.should_paginate_rules() {
            self.state.rules_visible_count = filtered_count;
            return;
        }
        if self.state.rules_visible_count == 0 {
            self.state.rules_visible_count = Self::RULES_PAGE_SIZE.min(filtered_count);
        } else if self.state.rules_visible_count > filtered_count {
            self.state.rules_visible_count = filtered_count;
        }
    }

    pub(super) fn try_lazy_load_rules_on_scroll(&mut self, cx: &mut Cx, event: &Event) {
        if self.state.active_page != Page::Rules || !self.should_paginate_rules() {
            return;
        }

        let Event::Scroll(_) = event else {
            return;
        };

        let area = self.ui.view(ids!(dashboard.content_body)).area();
        if area.is_empty() {
            return;
        }
        if !matches!(event.hits(cx, area), Hit::FingerScroll(_)) {
            return;
        }

        let filtered_count = self.filtered_rules().len();
        if filtered_count == 0 || self.state.rules_visible_count >= filtered_count {
            return;
        }

        self.state.rules_visible_count =
            (self.state.rules_visible_count + Self::RULES_PAGE_SIZE).min(filtered_count);
        self.refresh_ui(cx);
    }

    fn rule_matches_filter(rule: &str, filter: RuleFilter) -> bool {
        match filter {
            RuleFilter::All => true,
            RuleFilter::Domain => rule.starts_with("DOMAIN"),
            RuleFilter::IpCidr => rule.starts_with("IP-CIDR"),
            RuleFilter::ProcessName => rule.starts_with("PROCESS-NAME"),
        }
    }
}
