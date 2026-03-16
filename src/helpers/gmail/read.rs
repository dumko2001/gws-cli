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
use std::io::{self, Write};

/// Handle the `+read` subcommand.
pub(super) async fn handle_read(
    _doc: &crate::discovery::RestDescription,
    matches: &ArgMatches,
) -> Result<(), GwsError> {
    let message_id = matches.get_one::<String>("id").unwrap();

    let dry_run = matches.get_flag("dry-run");

    let original = if dry_run {
        OriginalMessage::dry_run_placeholder(message_id)
    } else {
        let t = auth::get_token(&[GMAIL_READONLY_SCOPE])
            .await
            .map_err(|e| GwsError::Auth(format!("Gmail auth failed: {e}")))?;

        let client = crate::client::build_client()?;
        fetch_message_metadata(&client, &t, message_id).await?
    };

    let format = matches.get_one::<String>("format").unwrap();
    let show_headers = matches.get_flag("headers");
    let use_html = matches.get_flag("html");

    let mut stdout = io::stdout().lock();

    if format == "json" {
        writeln!(
            stdout,
            "{}",
            serde_json::to_string_pretty(&original)
                .map_err(|e| GwsError::Other(anyhow::anyhow!(e)))?
        )
        .map_err(|e| GwsError::Other(anyhow::anyhow!(e)))?;
        return Ok(());
    }

    if show_headers {
        writeln!(stdout, "From: {}", original.from)
            .map_err(|e| GwsError::Other(anyhow::anyhow!(e)))?;
        writeln!(stdout, "To: {}", original.to).map_err(|e| GwsError::Other(anyhow::anyhow!(e)))?;
        if !original.cc.is_empty() {
            writeln!(stdout, "Cc: {}", original.cc)
                .map_err(|e| GwsError::Other(anyhow::anyhow!(e)))?;
        }
        writeln!(stdout, "Subject: {}", original.subject)
            .map_err(|e| GwsError::Other(anyhow::anyhow!(e)))?;
        writeln!(stdout, "Date: {}", original.date)
            .map_err(|e| GwsError::Other(anyhow::anyhow!(e)))?;
        writeln!(stdout, "---").map_err(|e| GwsError::Other(anyhow::anyhow!(e)))?;
    }

    let body = if use_html {
        original
            .body_html
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(&original.body_text)
    } else {
        &original.body_text
    };

    writeln!(stdout, "{}", body).map_err(|e| GwsError::Other(anyhow::anyhow!(e)))?;

    Ok(())
}
