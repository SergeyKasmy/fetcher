/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub struct MessageLengthLimiter<'a> {
	pub head: Option<&'a str>,
	pub body: Option<&'a str>,
	pub tail: Option<&'a str>,
}

impl MessageLengthLimiter<'_> {
	pub(crate) fn split_at(&mut self, max_len: usize) -> Option<String> {
		let should_insert_newline_after_head =
			self.head.is_some() && (self.body.is_some() || self.tail.is_some());
		let should_insert_newline_after_body = self.body.is_some() && self.tail.is_some();

		let msg_len = self.head.map_or(0, count_chars)
			+ self.body.map_or(0, count_chars)
			+ self.tail.map_or(0, count_chars)
			+ usize::from(should_insert_newline_after_head)
			+ usize::from(should_insert_newline_after_body);

		if msg_len == 0 {
			return None;
		}

		let next = if msg_len > max_len {
			compose_long_message(&mut self.head, &mut self.body, &mut self.tail, max_len)
		} else {
			Some(format!(
				"{}{}{}{}{}",
				self.head.take().unwrap_or_default(),
				should_insert_newline_after_head
					.then_some("\n")
					.unwrap_or_default(),
				self.body.take().unwrap_or_default(),
				should_insert_newline_after_body
					.then_some("\n")
					.unwrap_or_default(),
				self.tail.take().unwrap_or_default()
			))
		};

		assert!(
			next.as_ref().is_none_or(|s| !s.is_empty()),
			"We should've ensured the final composed message isn't empty but it is anyways"
		);

		next
	}
}

fn compose_long_message(
	head: &mut Option<&str>,
	body: &mut Option<&str>,
	tail: &mut Option<&str>,
	max_len: usize,
) -> Option<String> {
	if head.is_none() && body.is_none() && tail.is_none() {
		return None;
	}

	// make sure the entire head or tail can fit into the requested split
	// since they can't be split into parts
	let head_len = head.map_or(0, count_chars);
	assert!(
		max_len >= head_len,
		"head has more characters: {head_len}, than can be fit in a msg part of max len: {max_len}"
	);

	let tail_len = tail.map_or(0, count_chars);
	assert!(
		max_len >= tail_len,
		"tail has more characters: {tail_len}, than can be fit in a msg part of max len: {max_len}"
	);

	let mut split_part = String::with_capacity(max_len);

	// put the entire head into the split
	// should always fit because of the assertions up above
	if let Some(head) = head.take() {
		split_part.push_str(head);
	}

	if let Some(body_str) = body.take() {
		// find out how much space has remained for the body
		let space_left_for_body = max_len.checked_sub(split_part.chars().count()).expect(
			"only the head should've been pushed to the split and we asserted that it isn't longer than len",
		);

		// find the index at which point the body no longer fits into the split
		let body_fits_till = body_str
			.char_indices()
			.nth(space_left_for_body)
			// TODO: is .len() valid here or should it be .chars().count()?
			.map_or_else(|| body_str.len(), |(idx, _)| idx);

		// mark if we should add a newline character and leave some space for it
		let (body_fits_till, add_newline) = if split_part.is_empty() {
			(body_fits_till, false)
		} else {
			(body_fits_till.saturating_sub(1), true)
		};

		// if at least some of the body does fit
		if body_fits_till > 0 {
			// insert a new line to separate body from everything else
			if add_newline {
				split_part.push('\n');
			}

			split_part.push_str(&body_str[..body_fits_till]);

			// if there are some bytes remaining in the body, put them back into itself
			let remaining_body = &body_str[body_fits_till..];
			if !remaining_body.is_empty() {
				*body = Some(remaining_body);
			}
		} else {
			*body = Some(body_str);
		}
	}

	// tail
	{
		// mark if we should add a newline character and leave some space for it
		let (tail_len, add_newline) = if split_part.is_empty() {
			(tail_len, false)
		} else {
			(tail_len + 1, true)
		};

		// add the tail if it can still fit into the split
		if max_len.saturating_sub(split_part.chars().count()) >= tail_len {
			if let Some(tail) = tail.take() {
				// insert a newline to separate tail from everything else
				if add_newline {
					split_part.push('\n');
				}

				split_part.push_str(tail);
			}
		}
	}

	// make sure we haven't crossed our character limit
	{
		let split_part_chars = split_part.chars().count();
		assert!(
			split_part_chars <= max_len,
			"Returned a part with char len of {split_part_chars} when it should never be longer than {max_len}"
		);
	}

	Some(split_part)
}

fn count_chars(s: &str) -> usize {
	s.chars().count()
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;

	const MAX_MSG_LEN: usize = 4096;
	const BODY_COUNT: usize = 3;

	const HEAD: &str = "HEAD";
	const BODY: &str = "BODY";
	const TAIL: &str = "TAIL";

	impl Iterator for MessageLengthLimiter<'_> {
		type Item = String;

		fn next(&mut self) -> Option<Self::Item> {
			self.split_at(MAX_MSG_LEN)
		}
	}

	#[test]
	fn format_head_body_tail() {
		const FINAL: &str = "HEAD\nBODY\nTAIL";

		let mut msg = MessageLengthLimiter {
			head: Some(HEAD),
			body: Some(BODY),
			tail: Some(TAIL),
		};

		assert_eq!(msg.next().as_deref(), Some(FINAL));
		assert_eq!(msg.next(), None);
	}

	#[test]
	fn format_head_tail() {
		const FINAL: &str = "HEAD\nTAIL";

		let mut msg = MessageLengthLimiter {
			head: Some(HEAD),
			body: None,
			tail: Some(TAIL),
		};

		assert_eq!(msg.next().as_deref(), Some(FINAL));
		assert_eq!(msg.next(), None);
	}

	#[test]
	fn format_body_tail() {
		const FINAL: &str = "BODY\nTAIL";

		let mut msg = MessageLengthLimiter {
			head: None,
			body: Some(BODY),
			tail: Some(TAIL),
		};

		assert_eq!(msg.next().as_deref(), Some(FINAL));
		assert_eq!(msg.next(), None);
	}

	#[test]
	fn format_head_body() {
		const FINAL: &str = "HEAD\nBODY";

		let mut msg = MessageLengthLimiter {
			head: Some(HEAD),
			body: Some(BODY),
			tail: None,
		};

		assert_eq!(msg.next().as_deref(), Some(FINAL));
		assert_eq!(msg.next(), None);
	}

	#[test]
	fn short_body() {
		const STR: &str = "Hello, World!";

		let mut msg = MessageLengthLimiter {
			head: None,
			body: Some(STR),
			tail: None,
		};

		assert_eq!(msg.next().as_deref(), Some(STR));
		assert_eq!(msg.next(), None);
	}

	#[test]
	fn empty_head_tail_long_body() {
		let mut body = String::new();
		for _ in 0..MAX_MSG_LEN * BODY_COUNT {
			body.push('b');
		}

		let msg = MessageLengthLimiter {
			head: None,
			body: Some(&body),
			tail: None,
		};

		// check first msg is body[..MAX_MSG_LEN]
		let mut msg = msg.peekable();
		assert_eq!(msg.peek().map(|s| &**s), Some(&body[..MAX_MSG_LEN]));

		assert_eq!(msg.count(), BODY_COUNT);
	}

	#[test]
	fn long_head() {
		let mut head = String::new();
		for _ in 0..150 {
			head.push('h');
		}

		let mut body = String::new();
		for _ in 0..MAX_MSG_LEN * BODY_COUNT {
			body.push('b');
		}

		let msg = MessageLengthLimiter {
			head: Some(&head),
			body: Some(&body),
			tail: None,
		};

		// MSG_COUNT bodies + 1 head
		assert_eq!(msg.count(), BODY_COUNT + 1);
	}

	#[test]
	fn with_tail_almost_fitting() {
		let mut body = String::new();
		// body is 1 char from max msg len
		for _ in 0..MAX_MSG_LEN * BODY_COUNT - 1 {
			body.push('b');
		}

		let tail = "tt".to_owned(); // and tail is 2 char

		let msg = MessageLengthLimiter {
			head: None,
			body: Some(&body),
			tail: Some(&tail),
		};

		assert_eq!(msg.count(), BODY_COUNT + 1); // tail shouldn't be split and thus should be put into it's own msg
	}

	#[test]
	fn with_all_parts_of_max_len() {
		let mut head = String::new();
		for _ in 0..MAX_MSG_LEN {
			head.push('h');
		}

		let mut body = String::new();
		for _ in 0..MAX_MSG_LEN * BODY_COUNT {
			body.push('b');
		}

		let mut tail = String::new();
		for _ in 0..MAX_MSG_LEN {
			tail.push('t');
		}

		let msg = MessageLengthLimiter {
			head: Some(&head),
			body: Some(&body),
			tail: Some(&tail),
		};

		// MSG_COUNT bodies + 1 head & 1 tail
		assert_eq!(msg.count(), BODY_COUNT + 2);
	}

	// TODO: add test: .split_at shorter than minimum length (e.g. head length)
}
