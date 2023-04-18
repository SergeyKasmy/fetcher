/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod contains;
pub mod html;
pub mod take;

use self::{contains::ContainsState, take::TakeState};
use fetcher_config::jobs::action::Action;

use egui::{panel::Side, CentralPanel, ScrollArea, SelectableLabel, SidePanel, TopBottomPanel, Ui};
use std::{collections::HashMap, hash::Hash};

#[derive(Default, Debug)]
pub struct ActionsEditorState {
	pub current_action_idx: Option<usize>,
	pub selected_action_state: HashMap<usize, SelectedActionState>,
}

#[derive(Debug)]
pub enum SelectedActionState {
	Stateless,
	TakeState(TakeState),
	ContainsState(ContainsState),
}

impl ActionsEditorState {
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
			Action::Json(_) => todo!(),
			Action::Use(_) => todo!(),
			Action::Caps => Self::Stateless,
			Action::Set(_) => todo!(),
			Action::Shorten(_) => todo!(),
			Action::Trim(_) => todo!(),
			Action::Replace(_) => todo!(),
			Action::Extract(_) => todo!(),
			Action::RemoveHtml(_) => todo!(),
			Action::DecodeHtml(_) => todo!(),
			Action::Sink(_) => todo!(),
			Action::Import(_) => todo!(),
		}
	}

	pub fn show(&mut self, action: &mut Action, task_id: impl Hash, ui: &mut Ui) {
		match (self, action) {
			(Self::Stateless, Action::ReadFilter) => (),
			(Self::TakeState(state), Action::Take(x)) => state.show(x, &task_id, ui),
			(Self::ContainsState(state), Action::Contains(x)) => state.show(x, &task_id, ui),
			(Self::Stateless, Action::DebugPrint) => (),
			(Self::Stateless, Action::Feed) => (),
			(Self::Stateless, Action::Html(x)) => html::show(x, &task_id, ui),
			(_, Action::Http) => todo!(),
			(_, Action::Json(_)) => todo!(),
			(_, Action::Use(_)) => todo!(),
			(_, Action::Caps) => todo!(),
			(_, Action::Set(_)) => todo!(),
			(_, Action::Shorten(_)) => todo!(),
			(_, Action::Trim(_)) => todo!(),
			(_, Action::Replace(_)) => todo!(),
			(_, Action::Extract(_)) => todo!(),
			(_, Action::RemoveHtml(_)) => todo!(),
			(_, Action::DecodeHtml(_)) => todo!(),
			(_, Action::Sink(_)) => todo!(),
			(_, Action::Import(_)) => todo!(),
			_ => todo!(),
		}
	}
}
