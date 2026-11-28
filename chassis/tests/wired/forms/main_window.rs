//! The developer's handlers for the `MainWindow` form of the wiring test.

use crate::AppState;
use pushrod::engine::Ui;

/// Counts a click on the OK button.
pub fn on_ok_button_clicked(state: &mut AppState, _ui: &mut Ui<AppState>) {
    state.clicks += 1;
}

/// Remembers whether the Accept box is checked.
pub fn on_accept_toggled(state: &mut AppState, _ui: &mut Ui<AppState>, checked: bool) {
    state.accepted = checked;
}

/// Not an event handler: no widget and event are named like this, so it is never wired.
pub fn format_clicks(state: &AppState) -> String {
    format!("{} clicks", state.clicks)
}
