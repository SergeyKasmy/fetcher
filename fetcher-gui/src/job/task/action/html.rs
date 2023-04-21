/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_config::jobs::action::html::{
	query::{
		DataLocation, ElementAttr, ElementDataQuery, ElementKind, ElementQuery, HtmlQueryRegex,
		ItemQuery,
	},
	Html,
};

use egui::{ComboBox, Ui};
use std::hash::Hash;

pub fn show(html: &mut Html, task_id: impl Hash, ui: &mut Ui) {
	ui.collapsing("Item", |ui| {
		if let Some(item) = &mut html.item {
			element_queries(&mut item.query, ("item", &task_id), ui);

			if item.query.is_empty() {
				html.item = None;
			}
		}

		if html.item.is_none() && ui.button("+").clicked() {
			html.item = Some(ItemQuery {
				query: vec![ElementQuery::default()],
			});
		}
	});

	ui.collapsing("Title", |ui| {
		element_data_query(&mut html.title, ("title", &task_id), ui);
	});

	ui.collapsing("Text", |ui| {
		let query = &mut html.text;

		for (i, elem_data_queries) in query.iter_mut().enumerate() {
			for (j, elem_data_query) in elem_data_queries.iter_mut().enumerate() {
				if j > 0 {
					ui.separator();
				}

				element_data_query_inner(elem_data_query, ("text", &task_id, i, j), ui);
			}
		}

		ui.horizontal(|ui| {
			if ui.button("+").clicked() {
				query
					.get_or_insert_with(Default::default)
					.push(ElementDataQuery::default());
			}

			if ui.button("-").clicked() {
				if let Some(query_inner) = query {
					query_inner.pop();

					if query_inner.is_empty() {
						*query = None;
					}
				}
			}
		});
	});

	ui.collapsing("ID", |ui| {
		element_data_query(&mut html.id, ("id", &task_id), ui);
	});

	ui.collapsing("Link", |ui| {
		element_data_query(&mut html.link, ("link", &task_id), ui);
	});

	ui.collapsing("IMG", |ui| {
		element_data_query(&mut html.img, ("IMG", &task_id), ui);
	});
}

/// combined hash should be (task_id, query type, index)
fn element_data_query(
	elem_data_query: &mut Option<ElementDataQuery>,
	combined_hash: impl Hash,
	ui: &mut Ui,
) {
	if let Some(elem_data_query) = elem_data_query {
		element_data_query_inner(elem_data_query, combined_hash, ui);
	}

	match elem_data_query {
		None => {
			if ui.button("Add").clicked() {
				*elem_data_query = Some(ElementDataQuery::default());
			}
		}
		Some(_) => {
			if ui.button("Remove").clicked() {
				*elem_data_query = None;
			}
		}
	}
}

fn element_data_query_inner(
	elem_data_query: &mut ElementDataQuery,
	combined_hash: impl Hash,
	ui: &mut Ui,
) {
	ui.checkbox(&mut elem_data_query.optional, "optional");

	ui.group(|ui| element_queries(&mut elem_data_query.query, &combined_hash, ui));

	ui.horizontal(|ui| {
		ui.label("Data location:");
		ComboBox::from_id_source(("data location", &combined_hash))
			.selected_text(format!("{:?}", elem_data_query.data_location))
			.show_ui(ui, |ui| {
				ui.selectable_value(
					&mut elem_data_query.data_location,
					DataLocation::Text,
					"text",
				);

				ui.selectable_value(
					&mut elem_data_query.data_location,
					DataLocation::Attr(String::new()),
					"attr",
				);
			});

		if let DataLocation::Attr(attr) = &mut elem_data_query.data_location {
			ui.text_edit_singleline(attr);
		}
	});

	{
		let mut is_regex_enabled = elem_data_query.regex.is_some();

		ui.checkbox(&mut is_regex_enabled, "Regex replace");

		if is_regex_enabled {
			let HtmlQueryRegex { re, replace_with } =
				elem_data_query.regex.get_or_insert_with(Default::default);

			ui.horizontal(|ui| {
				ui.label("Regex");
				ui.text_edit_singleline(re);

				ui.label("Replace with");
				ui.text_edit_singleline(replace_with);
			});
		} else {
			elem_data_query.regex = None;
		}
	}
}

fn element_queries(elem_queries: &mut Vec<ElementQuery>, combined_hash: impl Hash, ui: &mut Ui) {
	for (idx, elem_query) in elem_queries.iter_mut().enumerate() {
		if idx > 0 {
			ui.separator();
		}

		element_query(elem_query, (idx, &combined_hash), ui);
	}

	ui.horizontal(|ui| {
		if ui.button("+").clicked() {
			elem_queries.push(ElementQuery::default());
		}

		if ui.button("-").clicked() {
			elem_queries.pop();
		}
	});
}

/// combined hash should be (task_id, query type, index)
fn element_query(elem_query: &mut ElementQuery, combined_hash: impl Hash, ui: &mut Ui) {
	element_kind(&mut elem_query.kind, &combined_hash, ui);

	ui.group(|ui| {
		ui.label("Ignore");
		for (idx, ignore) in elem_query.ignore.iter_mut().flatten().enumerate() {
			if idx > 0 {
				ui.separator();
			}

			element_kind(ignore, (&combined_hash, "ignore", idx), ui);
		}

		ui.horizontal(|ui| {
			if ui.button("+").clicked() {
				elem_query
					.ignore
					.get_or_insert_with(Vec::new)
					.push(ElementKind::default());
			}

			if ui.button("-").clicked() {
				if let Some(ignore) = &mut elem_query.ignore {
					ignore.pop();

					if ignore.is_empty() {
						elem_query.ignore = None;
					}
				}
			}
		});
	});
}

fn element_kind(elem_kind: &mut ElementKind, combined_hash: impl Hash, ui: &mut Ui) {
	ui.horizontal(|ui| {
		ui.label("Type:");
		ComboBox::from_id_source(("action html source element kind", combined_hash))
			.selected_text(format!("{elem_kind:?}"))
			.show_ui(ui, |ui| {
				ui.selectable_value(elem_kind, ElementKind::Tag(String::new()), "tag");
				ui.selectable_value(elem_kind, ElementKind::Class(String::new()), "class");
				ui.selectable_value(elem_kind, ElementKind::Attr(ElementAttr::default()), "attr");
			});

		match elem_kind {
			ElementKind::Tag(tag) => {
				ui.text_edit_singleline(tag);
			}
			ElementKind::Class(class) => {
				ui.text_edit_singleline(class);
			}
			ElementKind::Attr(ElementAttr { name, value }) => {
				ui.label("Name");
				ui.text_edit_singleline(name);

				ui.label("Value");
				ui.text_edit_singleline(value);
			}
		}
	});
}
