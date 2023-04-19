/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::COLOR_ERROR;

use super::field;
use fetcher_config::jobs::action::{shorten::Shorten, Field};

use egui::Ui;
use std::{collections::HashMap, hash::Hash};

#[derive(Default, Debug)]
pub struct ShortenState {
	pub number_field: HashMap<Field, String>,
	pub new_field: Field,
}

impl ShortenState {
	pub fn show(&mut self, shorten: &mut Shorten, task_id: impl Hash, ui: &mut Ui) {
		for (&field, shorten_to) in shorten.0.iter_mut() {
			ui.horizontal(|ui| {
				ui.label(field.to_string());

				let shorten_to_str = self
					.number_field
					.entry(field)
					.or_insert_with(|| shorten_to.to_string());

				ui.text_edit_singleline(shorten_to_str);

				match shorten_to_str.parse::<usize>() {
					Ok(v) => *shorten_to = v,
					Err(_) => {
						ui.colored_label(COLOR_ERROR, "Not a valid number");
					}
				}
			});
		}

		ui.horizontal(|ui| {
			if ui.button("+").clicked() {
				shorten.0.insert(self.new_field, 0);
			}

			field::show(
				&mut self.new_field,
				("action shorten edit field", &task_id),
				ui,
			);

			if ui.button("-").clicked() {
				shorten.0.remove(&self.new_field);
			}
		});
	}
}
