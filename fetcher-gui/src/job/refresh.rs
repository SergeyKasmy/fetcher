/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_config::jobs::job::timepoint::TimePoint;

use egui::Ui;

/// Saved state, not currently used state
#[derive(Debug)]
pub struct RefreshState {
	pub last_set: TimePoint,
	pub every: String,
	pub at: String,
}

impl RefreshState {
	pub fn show(&mut self, refresh: &mut Option<TimePoint>, ui: &mut Ui) {
		ui.horizontal(|ui| self.inner(refresh, ui));
	}

	/// main UI logic, just inside a separate method to remove an indent from ui.horizontal
	fn inner(&mut self, refresh: &mut Option<TimePoint>, ui: &mut Ui) {
		let mut is_refresh_enabled = refresh.is_some();
		ui.checkbox(&mut is_refresh_enabled, "refresh");

		if is_refresh_enabled {
			// if refresh is enabled, set it to last one set (or default)
			let refresh = refresh.get_or_insert_with(|| self.last_set.clone());

			// asked to change to TimePoint::At
			if ui
				.radio(matches!(refresh, TimePoint::At(_)), "at")
				.clicked()
			{
				match refresh {
					// it's currently TimePoint::Every, change it to TimePoint::At and use the saved state
					TimePoint::Every(s) => {
						// remember old state for later use
						self.every = s.clone();
						*refresh = TimePoint::At(self.at.clone());
					}
					TimePoint::At(_) => (),
				}
			}

			// asked to change to TimePoint::Every
			if ui
				.radio(matches!(refresh, TimePoint::Every(_)), "every")
				.clicked()
			{
				match refresh {
					// it's currently TimePoint::At, change it to TimePoint::Every and use the saved state
					TimePoint::At(s) => {
						// remember old state for later use
						self.at = s.clone();
						*refresh = TimePoint::Every(self.every.clone());
					}
					TimePoint::Every(_) => (),
				}
			}

			ui.text_edit_singleline(match refresh {
				TimePoint::Every(s) | TimePoint::At(s) => s,
			});
		} else {
			// remember last used refresh
			if let Some(refresh) = &refresh {
				self.last_set = refresh.clone();
			}

			*refresh = None;
		}
	}
}

impl Default for RefreshState {
	fn default() -> Self {
		Self {
			last_set: TimePoint::Every("30m".to_owned()),
			every: "30m".to_owned(),
			at: "12:00".to_owned(),
		}
	}
}
