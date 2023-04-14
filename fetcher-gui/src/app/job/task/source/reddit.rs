/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::app::{ScratchPad, COLOR_ERROR};
use fetcher_config::jobs::source::reddit::{self, Reddit};

use egui::{ComboBox, Ui};

pub fn show(ui: &mut Ui, Reddit(subreddits): &mut Reddit, scratch_pad: &mut ScratchPad) {
	for subreddit in subreddits.iter_mut() {
		let (
			name,
			reddit::Inner {
				sort,
				score_threshold,
			},
		) = subreddit;

		ui.heading(name);

		ComboBox::from_id_source(format!("source.reddit.{name}.sort"))
			.selected_text(format!("{sort:?}"))
			.show_ui(ui, |ui| {
				ui.selectable_value(sort, reddit::Sort::New, "new");
				ui.selectable_value(sort, reddit::Sort::Rising, "rising");
				ui.selectable_value(sort, reddit::Sort::Hot, "hot");
				ui.selectable_value(
					sort,
					reddit::Sort::Top(reddit::TimePeriod::Today),
					"top today",
				);
				ui.selectable_value(
					sort,
					reddit::Sort::Top(reddit::TimePeriod::ThisWeek),
					"top this week",
				);
				ui.selectable_value(
					sort,
					reddit::Sort::Top(reddit::TimePeriod::ThisMonth),
					"top this month",
				);
				ui.selectable_value(
					sort,
					reddit::Sort::Top(reddit::TimePeriod::ThisYear),
					"top this year",
				);
				ui.selectable_value(
					sort,
					reddit::Sort::Top(reddit::TimePeriod::AllTime),
					"top all time",
				);
			});

		let mut score_threshold_enabled = score_threshold.is_some();

		ui.checkbox(&mut score_threshold_enabled, "score threshold");

		ui.add_enabled_ui(score_threshold_enabled, |ui| {
			let score_threshold_str = scratch_pad
				.entry(format!("source.reddit.{name}.score_threshold"))
				.or_insert_with(|| {
					score_threshold
						.as_ref()
						.map(|x| x.to_string())
						.unwrap_or_else(|| String::from("0"))
				});

			ui.text_edit_singleline(score_threshold_str);

			if score_threshold_enabled {
				match score_threshold_str.parse::<u32>() {
					Ok(v) => *score_threshold = Some(v),
					Err(_) => {
						ui.colored_label(COLOR_ERROR, "Not a valid number");
					}
				}
			} else {
				*score_threshold = None;
			}
		});
	}

	ui.horizontal(|ui| {
		let edited_subreddit_name = scratch_pad
			.entry("source.reddit.edit".to_owned())
			.or_insert_with(|| "r/".to_owned());

		if ui.button("+").clicked() {
			subreddits
				.entry(edited_subreddit_name.to_owned())
				.or_default();
		}

		ui.text_edit_singleline(edited_subreddit_name);

		if ui.button("-").clicked() && !subreddits.is_empty() {
			subreddits.remove(edited_subreddit_name);
		}
	});
}
