/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod contains;
pub mod decode_html;
pub mod extract;
pub mod field;
pub mod html;
pub mod json;
pub mod remove_html;
pub mod replace;
pub mod set;
pub mod shorten;
pub mod sink;
pub mod take;
pub mod use_as;

use crate::get_state;

use self::{
	contains::ContainsState, json::JsonState, set::SetState, shorten::ShortenState,
	sink::SinkState, take::TakeState, use_as::UseState,
};
use fetcher_config::jobs::{
	action::{
		contains::Contains, decode_html::DecodeHtml, extract::Extract, html::Html, import::Import,
		json::Json, remove_html::RemoveHtml, replace::Replace, set::Set, shorten::Shorten,
		take::Take, trim::Trim, use_as::Use, Action,
	},
	sink::Sink,
};

use egui::{
	panel::Side, Align, Button, CentralPanel, ComboBox, Layout, ScrollArea, SelectableLabel,
	SidePanel, TopBottomPanel, Ui,
};
use std::hash::Hash;

#[derive(Default, Debug)]
pub struct ActionEditorState {
	pub current_action_idx: usize,
	// TODO: replace with vec and sync with actions vec to make removing actions not reset state
	pub selected_action_state: Vec<SelectedActionState>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum SelectedActionState {
	Stateless,
	TakeState(TakeState),
	ContainsState(ContainsState),
	JsonState(JsonState),
	UseState(UseState),
	SetState(SetState),
	ShortenState(ShortenState),
	SinkState(SinkState),
}

impl ActionEditorState {
	pub fn new(actions: Option<&[Action]>) -> Self {
		Self {
			current_action_idx: 0,
			selected_action_state: actions
				.into_iter()
				.flatten()
				.map(SelectedActionState::new)
				.collect(),
		}
	}

	pub fn show(&mut self, actions: &mut Option<Vec<Action>>, task_id: impl Hash, ui: &mut Ui) {
		SidePanel::new(Side::Left, egui::Id::new(("actions list", &task_id)))
			.show_inside(ui, |ui| self.side_panel(actions, &task_id, ui));

		// NOTE: fixes a bug in egui that makes the CentralPanel below overflow the window.
		// See https://github.com/emilk/egui/issues/901
		TopBottomPanel::bottom(egui::Id::new((
			"actions list invisible bottom panel",
			&task_id,
		)))
		.show_separator_line(false)
		.show_inside(ui, |_| ());

		CentralPanel::default().show_inside(ui, |ui| {
			ScrollArea::vertical().show(ui, |ui| {
				if let Some((state, act)) = actions.as_mut().and_then(|actions| {
					Some((
						self.selected_action_state
							.get_mut(self.current_action_idx)?,
						actions.get_mut(self.current_action_idx)?,
					))
				}) {
					state.show(act, task_id, ui);
				}
			});
		});
	}

	/// action list side panel + add button
	pub fn side_panel(
		&mut self,
		actions: &mut Option<Vec<Action>>,
		task_id: impl Hash,
		ui: &mut Ui,
	) {
		TopBottomPanel::bottom(egui::Id::new(("action list add button panel", &task_id)))
			.show_separator_line(false)
			.show_inside(ui, |ui| {
				ComboBox::from_id_source(("action list add button", &task_id))
					.selected_text("Add")
					.width(ui.available_width())
					.show_ui(ui, |ui| {
						/// Creates ui.selectable_label's for provided actions that pushes the action with the default state to the actions list
						macro_rules! add_button {
						    (
								$(
									Action::$act:ident
									$( => $default:expr)?
								),+
							) => {
								$({
									if ui.selectable_label(false, stringify!($act)).clicked() {
										let new_action = Action::$act$(($default))?; // push either Action::$act (for unit variants) or Action::$act($default) if the => $default arm is present

										self.selected_action_state.push(SelectedActionState::new(&new_action));

										actions
											.get_or_insert_with(Vec::new)
											.push(new_action);
									}
								})+
						    };
						}

						add_button! {
							Action::ReadFilter,
							Action::Take => Take::default(),
							Action::Contains => Contains::default(),
							Action::DebugPrint,
							Action::Feed,
							Action::Html => Html::default(),
							Action::Http,
							Action::Json => Json::default(),
							Action::Use => Use::default(),
							Action::Caps,
							Action::Set => Set::default(),
							Action::Shorten => Shorten::default(),
							Action::Trim => Trim::default(),
							Action::Replace => Replace::default(),
							Action::Extract => Extract::default(),
							Action::RemoveHtml => RemoveHtml::default(),
							Action::DecodeHtml => DecodeHtml::default(),
							Action::Sink => Sink::default(),
							Action::Import => Import::default()
						};
					});
			});

		let mut requested_to_delete_action = None;
		CentralPanel::default().show_inside(ui, |ui| {
			ScrollArea::vertical().show(ui, |ui| {
				for (idx, act) in actions.iter().flatten().enumerate() {
					ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
						/*
						if ui.button("-").clicked() {
							requested_to_delete_action = Some(idx);
						}


						ui.label(act.to_string());
						if ui
							.interact(
								ui.min_rect(),
								egui::Id::new(("action list item", idx, &task_id)),
								Sense::click(),
							)
							.clicked()
						{
							self.current_action_idx = Some(idx);
						}
						*/

						if ui.add(Button::new("-").frame(false)).clicked() {
							requested_to_delete_action = Some(idx);
						}

						// TODO: left align the text
						if ui
							.add_sized(
								[ui.available_width(), 0.0],
								SelectableLabel::new(
									self.current_action_idx == idx,
									act.to_string(),
								),
							)
							.clicked()
						{
							self.current_action_idx = idx;
						}
					});
				}
			});
		});

		if let Some((actions_vec, requested_to_delete_action)) =
			actions.as_mut().zip(requested_to_delete_action)
		{
			self.current_action_idx = self.current_action_idx.saturating_sub(1);

			self.selected_action_state
				.remove(requested_to_delete_action);

			actions_vec.remove(requested_to_delete_action);

			if actions_vec.is_empty() {
				*actions = None;
			}
		}
	}
}

impl SelectedActionState {
	pub fn new(for_action: &Action) -> Self {
		match for_action {
			Action::Take(_) => Self::TakeState(Default::default()),
			Action::Contains(_) => Self::ContainsState(Default::default()),
			Action::Json(_) => Self::JsonState(Default::default()),
			Action::Use(_) => Self::UseState(Default::default()),
			Action::Set(_) => Self::SetState(Default::default()),
			Action::Shorten(_) => Self::ShortenState(Default::default()),
			Action::Sink(_) => Self::SinkState(Default::default()),
			_ => Self::Stateless,
		}
	}

	pub fn show(&mut self, action: &mut Action, task_id: impl Hash, ui: &mut Ui) {
		match action {
			Action::Take(x) => get_state!(self, Self::TakeState).show(x, &task_id, ui),
			Action::Contains(x) => get_state!(self, Self::ContainsState).show(x, &task_id, ui),
			Action::Html(x) => html::show(x, &task_id, ui),
			Action::Json(x) => get_state!(self, Self::JsonState).show(x, &task_id, ui),
			Action::Use(x) => get_state!(self, Self::UseState).show(x, &task_id, ui),
			Action::Set(x) => get_state!(self, Self::SetState).show(x, &task_id, ui),
			Action::Shorten(x) => get_state!(self, Self::ShortenState).show(x, &task_id, ui),
			Action::Trim(x) => field::show(&mut x.field, &task_id, ui),
			Action::Replace(x) => replace::show(x, &task_id, ui),
			Action::Extract(x) => extract::show(x, &task_id, ui),
			Action::RemoveHtml(x) => remove_html::show(x, &task_id, ui),
			Action::DecodeHtml(x) => decode_html::show(x, &task_id, ui),
			Action::Sink(x) => get_state!(self, Self::SinkState).show(x, &task_id, ui),
			Action::Import(x) => {
				ui.text_edit_singleline(&mut x.0);
			}
			// other actions have no settings
			_ => (),
		}
	}
}
