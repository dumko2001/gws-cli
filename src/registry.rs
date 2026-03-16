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

//! Registry definitions for Personas, Recipes, and Helpers.

use serde::Deserialize;

pub const PERSONAS_YAML: &str = include_str!("../registry/personas.yaml");
pub const RECIPES_YAML: &str = include_str!("../registry/recipes.yaml");

/// Static metadata for a single helper command (e.g., `gws gmail +send`).
///
/// This table is the single source of truth for skill search and generation
/// filtering, avoiding runtime `clap::Command` construction just to read names.
/// It must be kept in sync with the `about` strings in each helper module.
pub struct HelperMeta {
    /// Primary service alias (matches `ServiceEntry::aliases[0]`).
    pub service_alias: &'static str,
    /// Short command name without the `+` prefix (e.g., `"send"`).
    pub name: &'static str,
    /// Human-readable description (from the `[Helper] …` `about` in the helper module).
    pub description: &'static str,
}

/// All registered helper commands in the order they appear in the CLI.
pub const HELPERS: &[HelperMeta] = &[
    // Gmail
    HelperMeta {
        service_alias: "gmail",
        name: "send",
        description: "Send an email",
    },
    HelperMeta {
        service_alias: "gmail",
        name: "triage",
        description: "Show unread inbox summary (sender, subject, date)",
    },
    HelperMeta {
        service_alias: "gmail",
        name: "reply",
        description: "Reply to a message (handles threading automatically)",
    },
    HelperMeta {
        service_alias: "gmail",
        name: "reply-all",
        description: "Reply-all to a message (handles threading automatically)",
    },
    HelperMeta {
        service_alias: "gmail",
        name: "forward",
        description: "Forward a message to new recipients",
    },
    HelperMeta {
        service_alias: "gmail",
        name: "watch",
        description: "Watch for new emails and stream them as NDJSON",
    },
    // Sheets
    HelperMeta {
        service_alias: "sheets",
        name: "append",
        description: "Append a row to a spreadsheet",
    },
    HelperMeta {
        service_alias: "sheets",
        name: "read",
        description: "Read values from a spreadsheet",
    },
    // Drive
    HelperMeta {
        service_alias: "drive",
        name: "upload",
        description: "Upload a file with automatic metadata",
    },
    // Docs
    HelperMeta {
        service_alias: "docs",
        name: "write",
        description: "Append text to a document",
    },
    // Calendar
    HelperMeta {
        service_alias: "calendar",
        name: "insert",
        description: "Create a new event",
    },
    HelperMeta {
        service_alias: "calendar",
        name: "agenda",
        description: "Show upcoming events across all calendars",
    },
    // Chat
    HelperMeta {
        service_alias: "chat",
        name: "send",
        description: "Send a message to a space",
    },
    // Workspace Events
    HelperMeta {
        service_alias: "events",
        name: "subscribe",
        description: "Subscribe to Workspace events and stream them as NDJSON",
    },
    HelperMeta {
        service_alias: "events",
        name: "renew",
        description: "Renew/reactivate Workspace Events subscriptions",
    },
    // Model Armor
    HelperMeta {
        service_alias: "modelarmor",
        name: "sanitize-prompt",
        description: "Sanitize a user prompt through a Model Armor template",
    },
    HelperMeta {
        service_alias: "modelarmor",
        name: "sanitize-response",
        description: "Sanitize a model response through a Model Armor template",
    },
    HelperMeta {
        service_alias: "modelarmor",
        name: "create-template",
        description: "Create a new Model Armor template",
    },
    // Workflow
    HelperMeta {
        service_alias: "workflow",
        name: "standup-report",
        description: "Today's meetings + open tasks as a standup summary",
    },
    HelperMeta {
        service_alias: "workflow",
        name: "meeting-prep",
        description: "Prepare for your next meeting: agenda, attendees, and linked docs",
    },
    HelperMeta {
        service_alias: "workflow",
        name: "email-to-task",
        description: "Convert a Gmail message into a Google Tasks entry",
    },
    HelperMeta {
        service_alias: "workflow",
        name: "weekly-digest",
        description: "Weekly summary: this week's meetings + unread email count",
    },
    HelperMeta {
        service_alias: "workflow",
        name: "file-announce",
        description: "Announce a Drive file in a Chat space",
    },
];

#[derive(Deserialize)]
pub struct PersonaRegistry {
    pub personas: Vec<PersonaEntry>,
}

#[derive(Deserialize)]
pub struct PersonaEntry {
    pub name: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub services: Vec<String>,
    #[serde(default)]
    pub workflows: Vec<String>,
    #[serde(default)]
    pub instructions: Vec<String>,
    #[serde(default)]
    pub tips: Vec<String>,
}

#[derive(Deserialize)]
pub struct RecipeRegistry {
    pub recipes: Vec<RecipeEntry>,
}

#[derive(Deserialize)]
pub struct RecipeEntry {
    pub name: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub services: Vec<String>,
    pub steps: Vec<String>,
    pub caution: Option<String>,
}
