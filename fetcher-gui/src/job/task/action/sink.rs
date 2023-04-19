/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::COLOR_ERROR;
use fetcher_config::jobs::sink::{discord, telegram};
use fetcher_config::jobs::sink::{Discord, Exec, Sink, Telegram};

use egui::{ComboBox, Ui};
use std::hash::Hash;

#[derive(Default, Debug)]
pub enum SinkState {
	TelegramState(TelegramState),
	DiscordState(DiscordState),
	#[default]
	Stateless,
}

#[derive(Default, Debug)]
pub struct TelegramState {
	pub chat_id_str: Option<String>,
}

#[derive(Default, Debug)]
pub struct DiscordState {
	pub id_str: Option<String>,
}

impl SinkState {
	pub fn show(&mut self, sink: &mut Sink, task_id: impl Hash, ui: &mut Ui) {
		ComboBox::from_id_source(("action sink", &task_id))
			.selected_text(format!("{sink:?}"))
			.show_ui(ui, |ui| {
				ui.selectable_value(sink, Sink::Telegram(Telegram::default()), "Telegram");
				ui.selectable_value(sink, Sink::Discord(Discord::default()), "Discord");
				ui.selectable_value(sink, Sink::Exec(Exec::default()), "Exec");
				ui.selectable_value(sink, Sink::Stdout, "stdout");
			});

		ui.separator();

		match sink {
			Sink::Telegram(x) => {
				// assert self is TelegramState or make replace self with TelegramState::default() otherwise
				match self {
					Self::TelegramState(_) => (),
					_ => *self = Self::TelegramState(Default::default()),
				}
				let Self::TelegramState(state) = self else {
					unreachable!("Should've replaced self with TelegramState in the code above");
				};

				state.show(x, task_id, ui);
			}
			Sink::Discord(x) => {
				// assert self is DiscordState or make replace self with DiscordState::default() otherwise
				match self {
					Self::DiscordState(_) => (),
					_ => *self = Self::DiscordState(Default::default()),
				}
				let Self::DiscordState(state) = self else {
					unreachable!("Should've replaced self with DiscordState in the code above");
				};

				state.show(x, ui);
			}
			Sink::Exec(x) => exec(x, ui),
			Sink::Stdout => (),
		}
	}
}

impl TelegramState {
	fn show(&mut self, telegram: &mut Telegram, task_id: impl Hash, ui: &mut Ui) {
		let chat_id_str = self
			.chat_id_str
			.get_or_insert_with(|| telegram.chat_id.to_string());

		ui.horizontal(|ui| {
			ui.label("chat_id");
			ui.text_edit_singleline(chat_id_str);

			match chat_id_str.parse::<i64>() {
				Ok(v) => telegram.chat_id = v,
				Err(_) => {
					ui.colored_label(COLOR_ERROR, "Not a valid number");
				}
			}
		});

		ui.horizontal(|ui| {
			ui.label("Link location");
			ComboBox::from_id_source(("sink telegram link location", task_id))
				.selected_text(format!("{:?}", telegram.link_location))
				.show_ui(ui, |ui| {
					ui.selectable_value(
						&mut telegram.link_location,
						telegram::LinkLocation::PreferTitle,
						"prefer title",
					);
					ui.selectable_value(
						&mut telegram.link_location,
						telegram::LinkLocation::Bottom,
						"bottom",
					);
				});
		});
	}
}

impl DiscordState {
	fn show(&mut self, discord: &mut Discord, ui: &mut Ui) {
		ui.horizontal(|ui| {
			if ui
				.radio(matches!(&discord.target, discord::Target::User(_)), "User")
				.clicked()
			{
				discord.target = discord::Target::User(0);
			}
			if ui
				.radio(
					matches!(&discord.target, discord::Target::Channel(_)),
					"Channel",
				)
				.clicked()
			{
				discord.target = discord::Target::Channel(0);
			}

			let id_str = self.id_str.get_or_insert_with(|| match discord.target {
				discord::Target::User(i) => i.to_string(),
				discord::Target::Channel(i) => i.to_string(),
			});

			ui.text_edit_singleline(id_str);

			match id_str.parse::<u64>() {
				Ok(v) => match &mut discord.target {
					discord::Target::User(i) => *i = v,
					discord::Target::Channel(i) => *i = v,
				},
				Err(_) => {
					ui.colored_label(COLOR_ERROR, "Not a valid number");
				}
			}
		});
	}
}

fn exec(exec: &mut Exec, ui: &mut Ui) {
	ui.horizontal(|ui| {
		ui.label("cmd:");
		ui.text_edit_singleline(&mut exec.cmd);
	});
}
