/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod discord;
pub mod email_password;
pub mod google;
pub mod telegram;
pub mod twitter;

pub use self::discord::Discord;
pub use self::email_password::EmailPassword;
pub use self::google::Google;
pub use self::telegram::Telegram;
pub use self::twitter::Twitter;
