#[derive(Clone, Debug)]
pub struct AppState {
    pub active_page: Page,
    pub language: Language,
    pub theme: ThemePreference,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Profiles,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    English,
    SimplifiedChinese,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemePreference {
    Light,
    Dark,
    System,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_page: Page::Profiles,
            language: Language::English,
            theme: ThemePreference::System,
        }
    }
}

impl Language {
    pub fn from_index(index: usize) -> Self {
        match index {
            1 => Self::SimplifiedChinese,
            _ => Self::English,
        }
    }

    pub fn as_index(self) -> usize {
        match self {
            Self::English => 0,
            Self::SimplifiedChinese => 1,
        }
    }
}

impl ThemePreference {
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Light,
            1 => Self::Dark,
            _ => Self::System,
        }
    }

    pub fn as_index(self) -> usize {
        match self {
            Self::Light => 0,
            Self::Dark => 1,
            Self::System => 2,
        }
    }
}
