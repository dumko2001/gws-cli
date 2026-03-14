// Copyright 2026 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;

/// Handle the `+read` subcommand.
pub(super) async fn handle_read(
    _doc: &crate::discovery::RestDescription,
    matches: &ArgMatches,
) -> Result<(), GwsError> {
    let message_id = matches.get_one::<String>("message-id").unwrap();

    let t = auth::get_token(&[GMAIL_SCOPE])
        .await
        .map_err(|e| GwsError::Auth(format!("Gmail auth failed: {e}")))?;
    
    let client = crate::client::build_client()?;
    let original = fetch_message_metadata(&client, &t, message_id).await?;

    println!("{}", original.body_text);

    Ok(())
}
