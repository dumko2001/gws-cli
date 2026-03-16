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
        let json_output = serde_json::to_string_pretty(&original)
            .context("Failed to serialize message to JSON")?;
        writeln!(stdout, "{}", json_output).context("Failed to write JSON output")?;
        return Ok(());
    }

    if show_headers {
        writeln!(stdout, "From: {}", sanitize_terminal_output(&original.from))
            .context("Failed to write 'From' header")?;
        writeln!(stdout, "To: {}", sanitize_terminal_output(&original.to))
            .context("Failed to write 'To' header")?;
        if !original.cc.is_empty() {
            writeln!(stdout, "Cc: {}", sanitize_terminal_output(&original.cc))
                .context("Failed to write 'Cc' header")?;
        }
        writeln!(
            stdout,
            "Subject: {}",
            sanitize_terminal_output(&original.subject)
        )
        .context("Failed to write 'Subject' header")?;
        writeln!(stdout, "Date: {}", sanitize_terminal_output(&original.date))
            .context("Failed to write 'Date' header")?;
        writeln!(stdout, "---").context("Failed to write header separator")?;
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

    writeln!(stdout, "{}", sanitize_terminal_output(body))
        .context("Failed to write message body")?;

    Ok(())
}

/// Sanitizes a string for terminal output by filtering out control characters
/// to prevent terminal injection attacks. Safe control characters like
/// newline, carriage return, and tab are preserved.
fn sanitize_terminal_output(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control() || matches!(c, '\n' | '\r' | '\t'))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_terminal_output() {
        let malicious = "Subject: \x1b]0;MALICIOUS\x07Hello\nWorld\r\t";
        let sanitized = sanitize_terminal_output(malicious);
        // ANSI escape sequences (control chars) should be removed
        assert!(!sanitized.contains('\x1b'));
        assert!(!sanitized.contains('\x07'));
        // Whitespace and formatting should be preserved
        assert!(sanitized.contains("Hello"));
        assert!(sanitized.contains('\n'));
        assert!(sanitized.contains('\r'));
        assert!(sanitized.contains('\t'));
    }
}
