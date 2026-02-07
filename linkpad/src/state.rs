#[derive(Clone, Debug)]
pub struct AppState {
    pub active_page: Page,
    pub menu: MenuState,
    pub profiles: PageContent,
    pub settings: PageContent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Profiles,
    Settings,
}

#[derive(Clone, Debug)]
pub struct MenuState {
    pub profiles_title: String,
    pub settings_title: String,
}

#[derive(Clone, Debug)]
pub struct PageContent {
    pub title: String,
    pub description: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self::mock()
    }
}

impl AppState {
    pub fn mock() -> Self {
        Self {
            active_page: Page::Profiles,
            menu: MenuState {
                profiles_title: "Profiles".to_string(),
                settings_title: "Settings".to_string(),
            },
            profiles: PageContent {
                title: "Profiles".to_string(),
                description: "Manage subscription profiles, local configs, and sync sources.".to_string(),
            },
            settings: PageContent {
                title: "Settings".to_string(),
                description: "App preferences, network options, and system integration.".to_string(),
            },
        }
    }

    pub fn active_content(&self) -> &PageContent {
        match self.active_page {
            Page::Profiles => &self.profiles,
            Page::Settings => &self.settings,
        }
    }
}
