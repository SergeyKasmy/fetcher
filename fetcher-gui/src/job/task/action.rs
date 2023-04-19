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

use self::{
	contains::ContainsState, json::JsonState, set::SetState, shorten::ShortenState,
	sink::SinkState, take::TakeState, use_as::UseState,
};
use fetcher_config::jobs::action::Action;

use egui::{panel::Side, CentralPanel, ScrollArea, SelectableLabel, SidePanel, TopBottomPanel, Ui};
use std::{collections::HashMap, hash::Hash};

#[derive(Default, Debug)]
pub struct ActionEditorState {
	pub current_action_idx: Option<usize>,
	pub selected_action_state: HashMap<usize, SelectedActionState>,
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
	pub fn show(&mut self, actions: &mut Option<Vec<Action>>, task_id: impl Hash, ui: &mut Ui) {
		SidePanel::new(Side::Left, egui::Id::new(("actions list", &task_id))).show_inside(
			ui,
			|ui| {
				ScrollArea::vertical().show(ui, |ui| {
					for (idx, act) in actions.iter().flatten().enumerate() {
						// TODO: left align the text
						if ui
							.add_sized(
								[ui.available_width(), 0.0],
								SelectableLabel::new(
									*self.current_action_idx.get_or_insert(0) == idx,
									act.to_string(),
								),
							)
							.clicked()
						{
							self.current_action_idx = Some(idx);
						}
					}
				});
			},
		);

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
				if let Some((idx, act)) = actions
					.as_mut()
					.zip(self.current_action_idx)
					.and_then(|(actions, idx)| Some((idx, actions.get_mut(idx)?)))
				{
					self.selected_action_state
						.entry(idx)
						.or_insert_with(|| SelectedActionState::new(act))
						.show(act, task_id, ui);
				}
			});
		});
	}
}

impl SelectedActionState {
	pub fn new(for_action: &Action) -> Self {
		match for_action {
			Action::ReadFilter => Self::Stateless,
			Action::Take(_) => Self::TakeState(Default::default()),
			Action::Contains(_) => Self::ContainsState(Default::default()),
			Action::DebugPrint => Self::Stateless,
			Action::Feed => Self::Stateless,
			Action::Html(_) => Self::Stateless,
			Action::Http => Self::Stateless,
			Action::Json(_) => Self::JsonState(Default::default()),
			Action::Use(_) => Self::UseState(Default::default()),
			Action::Caps => Self::Stateless,
			Action::Set(_) => Self::SetState(Default::default()),
			Action::Shorten(_) => Self::ShortenState(Default::default()),
			Action::Trim(_) => Self::Stateless,
			Action::Replace(_) => Self::Stateless,
			Action::Extract(_) => Self::Stateless,
			Action::RemoveHtml(_) => Self::Stateless,
			Action::DecodeHtml(_) => Self::Stateless,
			Action::Sink(_) => Self::SinkState(Default::default()),
			Action::Import(_) => Self::Stateless,
		}
	}

	pub fn show(&mut self, action: &mut Action, task_id: impl Hash, ui: &mut Ui) {
		match (&mut *self, &mut *action) {
			(Self::Stateless, Action::ReadFilter) => (),
			(Self::TakeState(state), Action::Take(x)) => state.show(x, &task_id, ui),
			(Self::ContainsState(state), Action::Contains(x)) => state.show(x, &task_id, ui),
			(Self::Stateless, Action::DebugPrint) => (),
			(Self::Stateless, Action::Feed) => (),
			(Self::Stateless, Action::Html(x)) => html::show(x, &task_id, ui),
			(Self::Stateless, Action::Http) => (),
			(Self::JsonState(state), Action::Json(x)) => state.show(x, &task_id, ui),
			(Self::UseState(state), Action::Use(x)) => state.show(x, &task_id, ui),
			(Self::Stateless, Action::Caps) => (),
			(Self::SetState(state), Action::Set(x)) => state.show(x, &task_id, ui),
			(Self::ShortenState(state), Action::Shorten(x)) => state.show(x, &task_id, ui),
			(Self::Stateless, Action::Trim(x)) => field::show(&mut x.field, &task_id, ui),
			(Self::Stateless, Action::Replace(x)) => replace::show(x, &task_id, ui),
			(Self::Stateless, Action::Extract(x)) => extract::show(x, &task_id, ui),
			(Self::Stateless, Action::RemoveHtml(x)) => remove_html::show(x, &task_id, ui),
			(Self::Stateless, Action::DecodeHtml(x)) => decode_html::show(x, &task_id, ui),
			(Self::SinkState(state), Action::Sink(x)) => state.show(x, &task_id, ui),
			(Self::Stateless, Action::Import(x)) => {
				ui.text_edit_singleline(&mut x.0);
			}
			// state doesn't match the action, create a new one
			_ => {
				/*
				// TODO: will create an infinite loop if no match arms still match. Create a check to avoid that
				*self = Self::new(action);
				self.show(action, task_id, ui);
				*/
				todo!();
			}
		}
	}
}
