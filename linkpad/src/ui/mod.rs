use makepad_components::makepad_widgets::Cx;

pub mod dashboard;
pub mod header;
pub mod sidebar;
pub mod style;

pub fn live_design(cx: &mut Cx) {
    style::live_design(cx);
    header::live_design(cx);
    sidebar::live_design(cx);
    dashboard::live_design(cx);
}
