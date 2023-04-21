/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::COLOR_ERROR;
use fetcher_config::jobs::action::json::{Json, JsonQueryRegex, Key, Query, StringQuery};

use egui::{ComboBox, Ui};
use std::{collections::HashMap, hash::Hash};

#[derive(Debug)]
pub struct JsonState {
	pub new_key_type: Key,
	pub item_usize_scratchpad: HashMap<usize, String>,
	pub title_usize_scratchpad: HashMap<usize, String>,
	pub text_usize_scratchpad: HashMap<usize, HashMap<usize, String>>,
	pub id_usize_scratchpad: HashMap<usize, String>,
	pub link_usize_scratchpad: HashMap<usize, String>,
	pub img_usize_scratchpad: HashMap<usize, HashMap<usize, String>>,
}

impl JsonState {
	pub fn show(&mut self, json: &mut Json, task_id: impl Hash, ui: &mut Ui) {
		ui.collapsing("Item", |ui| {
			query(
				&mut json.item,
				&mut self.new_key_type,
				&mut self.item_usize_scratchpad,
				("item", &task_id),
				ui,
			);
		});

		ui.collapsing("Title", |ui| {
			string_query(
				&mut json.title,
				&mut self.new_key_type,
				&mut self.title_usize_scratchpad,
				("title", &task_id),
				ui,
			);
		});

		ui.collapsing("Text", |ui| {
			for (idx, str_query) in json.text.iter_mut().flatten().enumerate() {
				keys(
					&mut str_query.query.keys,
					&mut self.new_key_type,
					self.text_usize_scratchpad.entry(idx).or_default(),
					("text", &task_id, idx),
					ui,
				);
			}

			ui.horizontal(|ui| {
				if ui.button("+").clicked() {
					json.text
						.get_or_insert_with(Default::default)
						.push(StringQuery::default());
				}

				if ui.button("-").clicked() {
					if let Some(text) = &mut json.text {
						text.pop();

						if text.is_empty() {
							json.text = None;
						}
					}
				}
			});
		});

		ui.collapsing("ID", |ui| {
			string_query(
				&mut json.id,
				&mut self.new_key_type,
				&mut self.id_usize_scratchpad,
				("id", &task_id),
				ui,
			);
		});

		ui.collapsing("Link", |ui| {
			string_query(
				&mut json.link,
				&mut self.new_key_type,
				&mut self.link_usize_scratchpad,
				("link", &task_id),
				ui,
			);
		});

		ui.collapsing("IMG", |ui| {
			for (idx, str_query) in json.img.iter_mut().flatten().enumerate() {
				keys(
					&mut str_query.query.keys,
					&mut self.new_key_type,
					self.img_usize_scratchpad.entry(idx).or_default(),
					("img", &task_id, idx),
					ui,
				);
			}

			ui.horizontal(|ui| {
				if ui.button("+").clicked() {
					json.img
						.get_or_insert_with(Default::default)
						.push(StringQuery::default());
				}

				if ui.button("-").clicked() {
					if let Some(img) = &mut json.img {
						img.pop();

						if img.is_empty() {
							json.img = None;
						}
					}
				}
			});
		});
	}
}

fn string_query(
	str_query: &mut Option<StringQuery>,
	new_key_type: &mut Key,
	usize_key_scratchpad: &mut HashMap<usize, String>,
	combined_hash: impl Hash,
	ui: &mut Ui,
) {
	if let Some(str_query) = str_query {
		query_inner(
			&mut str_query.query,
			new_key_type,
			usize_key_scratchpad,
			combined_hash,
			ui,
		);

		ui.horizontal(|ui| {
			let mut is_regex_enabled = str_query.regex.is_some();
			ui.checkbox(&mut is_regex_enabled, "Regex");

			if is_regex_enabled {
				let JsonQueryRegex { re, replace_with } =
					str_query.regex.get_or_insert_with(Default::default);

				ui.text_edit_singleline(re);
				ui.label("replace with");
				ui.text_edit_singleline(replace_with);
			} else {
				str_query.regex = None;
			}
		});
	}

	match str_query {
		None => {
			if ui.button("Add").clicked() {
				*str_query = Some(StringQuery::default());
			}
		}
		Some(_) => {
			if ui.button("Remove").clicked() {
				*str_query = None;
			}
		}
	}
}

fn query(
	query: &mut Option<Query>,
	new_key_type: &mut Key,
	usize_key_scratchpad: &mut HashMap<usize, String>,
	combined_hash: impl Hash,
	ui: &mut Ui,
) {
	if let Some(query) = query {
		query_inner(query, new_key_type, usize_key_scratchpad, combined_hash, ui);
	}

	match query {
		None => {
			if ui.button("Add").clicked() {
				*query = Some(Query::default());
			}
		}
		Some(_) => {
			if ui.button("Remove").clicked() {
				*query = None;
			}
		}
	}
}

fn query_inner(
	query: &mut Query,
	new_key_type: &mut Key,
	usize_key_scratchpad: &mut HashMap<usize, String>,
	combined_hash: impl Hash,
	ui: &mut Ui,
) {
	ui.group(|ui| {
		keys(
			&mut query.keys,
			new_key_type,
			usize_key_scratchpad,
			combined_hash,
			ui,
		);
	});

	ui.checkbox(&mut query.optional, "optional");
}

fn keys(
	keys: &mut Vec<Key>,
	new_key_type: &mut Key,
	usize_key_scratchpad: &mut HashMap<usize, String>,
	combined_hash: impl Hash,
	ui: &mut Ui,
) {
	ui.label("Keys");

	for (idx, key) in keys.iter_mut().enumerate() {
		match key {
			Key::String(s) => {
				ui.text_edit_singleline(s);
			}
			Key::Usize(i) => {
				let usize_s = usize_key_scratchpad
					.entry(idx)
					.or_insert_with(|| i.to_string());

				ui.text_edit_singleline(usize_s);

				match usize_s.parse::<usize>() {
					Ok(v) => {
						*i = v;
					}
					Err(_) => {
						ui.colored_label(COLOR_ERROR, "Not a valid number");
					}
				}
			}
		}
	}

	ui.horizontal(|ui| {
		if ui.button("+").clicked() {
			keys.push(new_key_type.clone());
		}

		ComboBox::from_id_source(("action json new key type", &combined_hash))
			.selected_text(format!("{:?}", new_key_type))
			.show_ui(ui, |ui| {
				ui.selectable_value(new_key_type, Key::String(String::new()), "String key");
				ui.selectable_value(new_key_type, Key::Usize(0), "Number key");
			});

		if ui.button("-").clicked() {
			keys.pop();
		}
	});
}

impl Default for JsonState {
	fn default() -> Self {
		Self {
			new_key_type: Key::String(String::new()),
			item_usize_scratchpad: Default::default(),
			title_usize_scratchpad: Default::default(),
			text_usize_scratchpad: Default::default(),
			id_usize_scratchpad: Default::default(),
			link_usize_scratchpad: Default::default(),
			img_usize_scratchpad: Default::default(),
		}
	}
}
