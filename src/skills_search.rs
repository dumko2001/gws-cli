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

//! Search and discovery for agent skills.

use crate::discovery;
use crate::error::GwsError;
use crate::helpers;
use crate::registry::{PersonaRegistry, RecipeRegistry, PERSONAS_YAML, RECIPES_YAML};
use crate::services;
use clap::Command;

/// Maps common natural-language terms to the canonical field values used in skill metadata.
///
/// When a user searches for "email", this table expands the token to also include "gmail",
/// so `gws skills search email` finds the Gmail service even though "email" does not appear
/// literally in its `api_name` or `aliases`. Each entry is `(user_term, canonical_expansion)`.
///
/// Expansions are additive — the original token is always kept — so an exact match on the
/// canonical name still works even without a synonym entry.
const SYNONYMS: &[(&str, &str)] = &[
    // Product / service synonyms
    ("email", "gmail"),
    ("mail", "gmail"),
    ("inbox", "gmail"),
    ("spreadsheet", "sheets"),
    ("excel", "sheets"),
    ("sheet", "sheets"),
    ("schedule", "calendar"),
    ("meeting", "calendar"),
    ("event", "calendar"),
    ("document", "docs"),
    ("word", "docs"),
    ("doc", "docs"),
    ("presentation", "slides"),
    ("powerpoint", "slides"),
    ("deck", "slides"),
    ("storage", "drive"),
    ("file", "drive"),
    ("folder", "drive"),
    ("note", "keep"),
    ("task", "tasks"),
    ("todo", "tasks"),
    ("contact", "people"),
    ("address", "people"),
    ("video", "meet"),
    ("conference", "meet"),
    ("survey", "forms"),
    ("course", "classroom"),
    ("audit", "admin-reports"),
    ("log", "admin-reports"),
    ("safety", "modelarmor"),
    ("automation", "workflow"),
];

/// Expands each query token via the synonym table, returning the original tokens plus
/// any canonical expansions. Duplicates are deduplicated. The expanded set is used so
/// that searching "email" also matches fields containing "gmail".
pub(crate) fn expand_tokens(tokens: &[String]) -> Vec<String> {
    let mut expanded: Vec<String> = tokens.to_vec();
    for token in tokens {
        for (term, canonical) in SYNONYMS {
            if token == term && !expanded.iter().any(|e| e == canonical) {
                expanded.push(canonical.to_string());
            }
        }
    }
    expanded
}

fn print_skills_help() {
    println!("USAGE:");
    println!("    gws skills search <query>");
    println!();
    println!("DESCRIPTION:");
    println!("    Search for agent skills by name or description.");
    println!("    Searches across services, helpers, personas, and recipes.");
    println!("    Common synonyms are expanded automatically (e.g. \"email\" finds Gmail).");
    println!();
    println!("ARGUMENTS:");
    println!("    <query>    Keywords to search for (multi-word queries are supported)");
    println!();
    println!("EXAMPLES:");
    println!("    gws skills search email");
    println!("    gws skills search \"send email\"");
    println!("    gws skills search upload file");
    println!("    gws skills search spreadsheet");
}

/// Entry point for `gws skills search <query>`.
pub async fn handle_skills_command(args: &[String]) -> Result<(), GwsError> {
    if args.is_empty() || args[0] != "search" {
        // `gws skills` or `gws skills --help` → show help (don't error on bare invocation)
        print_skills_help();
        return Ok(());
    }

    // Handle `gws skills search --help`
    if args.len() >= 2 && (args[1].as_str() == "--help" || args[1].as_str() == "-h") {
        print_skills_help();
        return Ok(());
    }

    if args.len() < 2 {
        return Err(GwsError::Validation(
            "No search query provided. Usage: gws skills search <query>".to_string(),
        ));
    }

    // Split into individual tokens so multi-word queries like "send email" match
    // descriptions where the words appear separately (e.g. "Send an email").
    // We split each argument by whitespace to handle cases where the user quoted
    // their query (e.g. `gws skills search "send email"`).
    let mut raw_tokens = Vec::new();
    for arg in &args[1..] {
        for token in arg.split_whitespace() {
            raw_tokens.push(token.to_lowercase());
        }
    }
    let query_display = raw_tokens.join(" ");

    println!("Searching for skills matching \"{}\"...\n", query_display);

    let mut results = 0;

    // For each raw token, pre-compute the full candidate set: the token itself plus all
    // synonym expansions. e.g. "email" → ["email", "gmail"].
    // matches() then requires ALL raw tokens to have at least one candidate appear in the
    // combined fields (token-AND with synonym-OR per token).
    let token_candidates: Vec<Vec<String>> = raw_tokens
        .iter()
        .map(|t| expand_tokens(std::slice::from_ref(t)))
        .collect();

    let matches = |fields: &[&str]| -> bool {
        let combined = fields.join(" ").to_lowercase();
        token_candidates
            .iter()
            .all(|candidates| candidates.iter().any(|c| combined.contains(c.as_str())))
    };

    // Search Services
    for svc in services::SERVICES {
        let fields: Vec<&str> = std::iter::once(svc.api_name)
            .chain(std::iter::once(svc.description))
            .chain(svc.aliases.iter().copied())
            .collect();
        if matches(&fields) {
            println!("[Service] gws-{} - {}", svc.aliases[0], svc.description);
            println!(
                "  Reference: skills/references/gws-{}/SKILL.md\n",
                svc.aliases[0]
            );
            results += 1;
        }
    }

    // Search Helpers
    for svc in services::SERVICES {
        if let Some(helper) = helpers::get_helper(svc.api_name) {
            let cli = Command::new(svc.api_name);
            let doc = discovery::RestDescription {
                name: svc.api_name.to_string(),
                ..Default::default()
            };
            let cli_with_helpers = helper.inject_commands(cli, &doc);
            for sub in cli_with_helpers.get_subcommands() {
                let name = sub.get_name();
                if name.starts_with('+') {
                    let short_name = name.trim_start_matches('+');
                    let full_helper_name = format!("gws-{}-{}", svc.aliases[0], short_name);
                    let about = sub.get_about().map(|s| s.to_string()).unwrap_or_default();
                    let about_clean = about.strip_prefix("[Helper] ").unwrap_or(&about);

                    if matches(&[full_helper_name.as_str(), about_clean]) {
                        println!("[Helper] {} - {}", full_helper_name, about_clean);
                        println!(
                            "  Reference: skills/references/{}/SKILL.md\n",
                            full_helper_name
                        );
                        results += 1;
                    }
                }
            }
        }
    }

    // Search Personas
    let persona_registry: PersonaRegistry = serde_yaml::from_str(PERSONAS_YAML)
        .map_err(|e| GwsError::Validation(format!("Failed to parse personas.yaml: {e}")))?;
    for p in persona_registry.personas {
        if matches(&[p.name.as_str(), p.title.as_str(), p.description.as_str()]) {
            println!("[Persona] persona-{} - {}", p.name, p.title);
            println!("  Description: {}", p.description);
            println!("  Skill: skills/persona-{}/SKILL.md\n", p.name);
            results += 1;
        }
    }

    // Search Recipes
    let recipe_registry: RecipeRegistry = serde_yaml::from_str(RECIPES_YAML)
        .map_err(|e| GwsError::Validation(format!("Failed to parse recipes.yaml: {e}")))?;
    for r in recipe_registry.recipes {
        if matches(&[r.name.as_str(), r.title.as_str(), r.description.as_str()]) {
            println!("[Recipe] recipe-{} - {}", r.name, r.title);
            println!("  Description: {}", r.description);
            println!("  Skill: skills/recipe-{}/SKILL.md\n", r.name);
            results += 1;
        }
    }

    if results == 0 {
        println!("No matching skills found.");
    } else {
        println!("Found {} matching skills.", results);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tokens_no_synonym() {
        let tokens = vec!["gmail".to_string()];
        let expanded = expand_tokens(&tokens);
        assert_eq!(expanded, vec!["gmail"]);
    }

    #[test]
    fn test_expand_tokens_email_expands_to_gmail() {
        let tokens = vec!["email".to_string()];
        let expanded = expand_tokens(&tokens);
        assert!(expanded.contains(&"email".to_string()));
        assert!(expanded.contains(&"gmail".to_string()));
    }

    #[test]
    fn test_expand_tokens_spreadsheet_expands_to_sheets() {
        let tokens = vec!["spreadsheet".to_string()];
        let expanded = expand_tokens(&tokens);
        assert!(expanded.contains(&"sheets".to_string()));
    }

    #[test]
    fn test_expand_tokens_no_duplicates() {
        // "sheet" and "sheets" both expand to "sheets", but "sheets" should appear only once
        let tokens = vec!["sheet".to_string(), "sheets".to_string()];
        let expanded = expand_tokens(&tokens);
        let sheets_count = expanded.iter().filter(|t| t.as_str() == "sheets").count();
        assert_eq!(sheets_count, 1);
    }

    #[test]
    fn test_expand_tokens_multi_word() {
        let tokens = vec!["send".to_string(), "email".to_string()];
        let expanded = expand_tokens(&tokens);
        assert!(expanded.contains(&"send".to_string()));
        assert!(expanded.contains(&"email".to_string()));
        assert!(expanded.contains(&"gmail".to_string()));
    }

    #[test]
    fn test_expand_tokens_preserves_original() {
        // Even after expansion, the original token must be retained
        let tokens = vec!["mail".to_string()];
        let expanded = expand_tokens(&tokens);
        assert!(expanded.contains(&"mail".to_string()));
        assert!(expanded.contains(&"gmail".to_string()));
    }

    #[test]
    fn test_synonyms_table_no_duplicate_terms() {
        // Each (term, canonical) pair should be unique to avoid redundant expansions
        let mut seen = std::collections::HashSet::new();
        for (term, canonical) in SYNONYMS {
            let key = (*term, *canonical);
            assert!(
                seen.insert(key),
                "Duplicate synonym entry: ({term}, {canonical})"
            );
        }
    }

    #[test]
    fn test_synonyms_all_canonicals_are_lowercase() {
        for (_, canonical) in SYNONYMS {
            assert_eq!(
                *canonical,
                canonical.to_lowercase(),
                "Canonical '{canonical}' is not lowercase"
            );
        }
    }
}
