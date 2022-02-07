/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

// FIXME
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

pub(crate) mod formats;

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use teloxide::Bot;
// use toml::{value::Map, Value};

use crate::{
	config::formats::TwitterCfg,
	error::Error,
	error::Result,
	settings,
	sink::{Sink, Telegram},
	source::{
		email::Filters as EmailFilters, email::ViewMode as EmailViewMode, Email, Rss, Source,
		Twitter,
	},
};

#[derive(Debug)]
pub struct Config {
	pub name: String,
	pub source: Source,
	pub sink: Sink,
	pub refresh: u64,
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct Entries(HashMap<String, Entry>);

#[derive(Deserialize, Debug)]
pub struct Entry {
	disabled: Option<bool>,
	refresh: u64,
	chat_id: i64,
	#[serde(flatten)]
	value: Value,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Value {
	Rss {
		url: String,
	},
	Twitter {
		filter: Vec<String>,
		pretty_name: String,
		handle: String,
	},
	Email {
		pretty_name: String,
		email: String,
		imap: String,
		auth_type: EmailAuthType,
		view_mode: EmailViewMode,
		footer: String,
	},
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum EmailAuthType {
	#[serde(alias = "google_oauth2")]
	GoogleOAuth2,
	Password,
}

impl Config {
	pub async fn parse(conf_raw: &str) -> Result<Vec<Self>> {
		todo!()
	}
}
