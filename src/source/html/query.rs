#[derive(Clone, Debug)]
pub(crate) enum QueryKind {
	Tag { value: String },
	Class { value: String },
	Attr { name: String, value: String },
}

#[derive(Debug)]
pub(crate) enum DataLocation {
	Text,
	Attr { value: String },
}

#[derive(Debug)]
pub(crate) struct Query {
	pub(crate) kind: Vec<QueryKind>,
	pub(crate) data_location: DataLocation,
}

#[derive(Debug)]
pub(crate) struct TextQuery {
	pub(crate) prepend: Option<String>,
	pub(crate) inner: Query,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum IdQueryKind {
	String,
	Date,
}

#[derive(Debug)]
pub(crate) struct IdQuery {
	pub(crate) kind: IdQueryKind,
	pub(crate) inner: Query,
}

#[derive(Debug)]
pub(crate) struct LinkQuery {
	pub(crate) prepend: Option<String>,
	pub(crate) inner: Query,
}

#[derive(Debug)]
pub(crate) struct ImageQuery {
	pub(crate) optional: bool,
	pub(crate) inner: LinkQuery,
}
