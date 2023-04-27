/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::COLOR_ERROR;
use fetcher_config::jobs::source::http::{self, Http, Url};

use egui::{ComboBox, Ui};
use once_cell::sync::Lazy;
use std::{collections::HashMap, str::FromStr};

static EXAMPLE_URL: Lazy<Url> =
	Lazy::new(|| Url::try_from("http://example.com").expect("example.com should be a valid URL"));

#[derive(Default, Debug)]
pub struct HttpState {
	pub urls: HashMap<usize, String>,
}

impl HttpState {
	pub fn show(&mut self, Http(requests): &mut Http, ui: &mut Ui) {
		for (idx, request) in requests.iter_mut().enumerate() {
			if idx > 0 {
				ui.separator();
			}

			match request {
				// GET
				http::Request::Untagged(url)
				| http::Request::Tagged(http::TaggedRequest::Get(url)) => {
					let scratch_pad = self.urls.entry(idx).or_insert_with(|| url.to_string());

					ui.heading("GET");
					edit_url(ui, url, scratch_pad);
				}

				// POST
				http::Request::Tagged(http::TaggedRequest::Post { url, body }) => {
					let scratch_pad = self.urls.entry(idx).or_insert_with(|| url.to_string());

					ui.heading("POST");
					edit_url(ui, url, scratch_pad);

					ui.horizontal(|ui| {
						ui.label("Request:");
						ui.text_edit_multiline(body);
					});
				}
			}
		}

		if !requests.is_empty() {
			ui.separator();
		}

		ui.horizontal(|ui| {
			ComboBox::from_id_source("add http request")
				.selected_text("+")
				.show_ui(ui, |combo| {
					if combo.selectable_label(false, "GET").clicked() {
						requests.push(http::Request::Tagged(http::TaggedRequest::Get(
							EXAMPLE_URL.clone(),
						)));
					}

					if combo.selectable_label(false, "POST").clicked() {
						requests.push(http::Request::Tagged(http::TaggedRequest::Post {
							url: EXAMPLE_URL.clone(),
							body: String::new(),
						}));
					}
				});

			if ui.button("-").clicked() && !requests.is_empty() {
				requests.remove(requests.len() - 1);
			}
		});
	}
}

fn edit_url(ui: &mut egui::Ui, url: &mut Url, scratch_pad: &mut String) {
	let url_str = scratch_pad;

	ui.horizontal(|ui| {
		ui.label("URL:");
		ui.text_edit_singleline(url_str);
	});

	match Url::from_str(url_str) {
		Ok(new_url) => {
			*url = new_url;
		}
		Err(_) => {
			ui.colored_label(COLOR_ERROR, "Invalid URL");
		}
	}
}
