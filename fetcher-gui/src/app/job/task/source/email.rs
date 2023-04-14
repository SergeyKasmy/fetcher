/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::app::ScratchPad;
use fetcher_config::jobs::source::email::{Auth, Email, ViewMode};

use egui::{ComboBox, Ui};

pub fn show(ui: &mut Ui, email: &mut Email, scratch_pad: &mut ScratchPad) {
	let Email {
		imap,
		email,
		auth,
		filters,
		view_mode,
	} = email;

	let mut imap_enabled = imap.is_some();

	ui.checkbox(&mut imap_enabled, "imap");

	ui.add_enabled_ui(imap_enabled, |ui| {
		let imap_str = scratch_pad
			.entry("source.email.imap".to_owned())
			.or_insert_with(|| imap.clone().unwrap_or_default());

		ui.horizontal(|ui| {
			ui.label("IMAP server:");
			ui.text_edit_singleline(imap_str);
		});

		if imap_enabled {
			*imap = Some(imap_str.clone());
		} else {
			*imap = None;
		}
	});

	ui.horizontal(|ui| {
		ui.label("Email:");
		ui.text_edit_singleline(email);
	});

	ComboBox::from_id_source("email source auth type")
		.selected_text(format!("{auth:?}"))
		.show_ui(ui, |ui| {
			ui.selectable_value(auth, Auth::GmailOAuth2, "Gmail OAuth2");
			ui.selectable_value(auth, Auth::Password, "Password");
		});

	ui.label("Filters");
	ui.group(|ui| {
		{
			let mut sender_enabled = filters.sender.is_some();

			ui.checkbox(&mut sender_enabled, "sender");

			ui.add_enabled_ui(sender_enabled, |ui| {
				let sender_str = scratch_pad
					.entry("source.email.filters.sender".to_owned())
					.or_insert_with(|| filters.sender.clone().unwrap_or_default());

				ui.horizontal(|ui| {
					ui.label("Sender:");
					ui.text_edit_singleline(sender_str);
				});

				if sender_enabled {
					filters.sender = Some(sender_str.clone());
				} else {
					filters.sender = None;
				}
			});
		}

		{
			let mut subjects_enabled = filters.subjects.is_some();

			ui.checkbox(&mut subjects_enabled, "subjects");

			ui.add_enabled_ui(subjects_enabled, |ui| {
				for (idx, subject) in filters.subjects.iter_mut().flatten().enumerate() {
					let subject_str = scratch_pad
						.entry(format!("source.email.filters.subjects.{idx}"))
						.or_insert_with(|| subject.clone());

					ui.horizontal(|ui| {
						ui.label("Subject:");
						ui.text_edit_singleline(subject_str);
					});

					*subject = subject_str.clone();
				}

				if subjects_enabled {
					ui.horizontal(|ui| {
						let subjects = filters.subjects.get_or_insert_with(Vec::new);

						if ui.button("+").clicked() {
							subjects.push(String::new());
						}

						if ui.button("-").clicked() && !subjects.is_empty() {
							match &mut filters.subjects {
								Some(v) => {
									v.remove(v.len() - 1);
									if v.is_empty() {
										filters.subjects = None;
									}
								}
								None => (),
							}
						}
					});
				} else {
					filters.subjects = None;
				}
			});
		}

		{
			let mut exclude_subjects_enabled = filters.exclude_subjects.is_some();

			ui.checkbox(&mut exclude_subjects_enabled, "exclude subjects");

			ui.add_enabled_ui(exclude_subjects_enabled, |ui| {
				for (idx, exclude_subject) in
					filters.exclude_subjects.iter_mut().flatten().enumerate()
				{
					let exclude_subject_str = scratch_pad
						.entry(format!("source.email.filters.exclude_subjects.{idx}"))
						.or_insert_with(|| exclude_subject.clone());

					ui.horizontal(|ui| {
						ui.label("Exclude subject:");
						ui.text_edit_singleline(exclude_subject_str);
					});

					*exclude_subject = exclude_subject_str.clone();
				}

				if exclude_subjects_enabled {
					ui.horizontal(|ui| {
						let exclude_subjects =
							filters.exclude_subjects.get_or_insert_with(Vec::new);

						if ui.button("+").clicked() {
							exclude_subjects.push(String::new());
						}

						if ui.button("-").clicked() && !exclude_subjects.is_empty() {
							match &mut filters.exclude_subjects {
								Some(v) => {
									v.remove(v.len() - 1);
									if v.is_empty() {
										filters.exclude_subjects = None;
									}
								}
								None => (),
							}
						}
					});
				} else {
					filters.exclude_subjects = None;
				}
			});
		}
	});

	ui.horizontal(|ui| {
		ui.label("View Mode");

		ComboBox::from_id_source("email view mode")
			.selected_text(format!("{view_mode:?}"))
			.show_ui(ui, |ui| {
				ui.selectable_value(view_mode, ViewMode::Delete, "delete");
				ui.selectable_value(view_mode, ViewMode::MarkAsRead, "mark as read");
				ui.selectable_value(view_mode, ViewMode::ReadOnly, "read only");
			});
	});
}
