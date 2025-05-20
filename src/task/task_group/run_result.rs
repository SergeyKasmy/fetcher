use crate::error::FetcherError;

pub trait RunResult: IntoIterator<Item = Result<(), FetcherError>> {}

impl<const N: usize> RunResult for [Result<(), FetcherError>; N] {}
impl RunResult for Vec<Result<(), FetcherError>> {}
impl RunResult for std::iter::Once<Result<(), FetcherError>> {}
