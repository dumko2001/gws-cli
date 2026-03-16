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

use crate::error::GwsError;
use crate::registry::{PersonaRegistry, RecipeRegistry, HELPERS, PERSONAS_YAML, RECIPES_YAML};
use crate::services;

fn print_skills_help() {
    println!("USAGE:");
    println!("    gws skills search <query>");
    println!();
    println!("DESCRIPTION:");
    println!("    Search for agent skills by name or description.");
    println!("    Searches across services, helpers, personas, and recipes.");
    println!();
    println!("ARGUMENTS:");
    println!("    <query>    Keywords to search for (multi-word queries are supported)");
    println!();
    println!("EXAMPLES:");
    println!("    gws skills search email");
    println!("    gws skills search \"send email\"");
    println!("    gws skills search upload file");
}

/// Returns `true` when every token in `query_tokens` appears somewhere in the
/// space-joined, lower-cased `fields`.
pub(crate) fn token_matches(query_tokens: &[String], fields: &[&str]) -> bool {
    let combined = fields.join(" ").to_lowercase();
    query_tokens.iter().all(|t| combined.contains(t.as_str()))
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
    let query_tokens: Vec<String> = args[1..].iter().map(|a| a.to_lowercase()).collect();
    let query_display = query_tokens.join(" ");

    println!("Searching for skills matching \"{}\"...\n", query_display);

    let mut results = 0;

    let matches = |fields: &[&str]| token_matches(&query_tokens, fields);

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
    for h in HELPERS {
        let full_helper_name = format!("gws-{}-{}", h.service_alias, h.name);
        if matches(&[full_helper_name.as_str(), h.description]) {
            println!("[Helper] {} - {}", full_helper_name, h.description);
            println!(
                "  Reference: skills/references/{}/SKILL.md\n",
                full_helper_name
            );
            results += 1;
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

    fn tokens(s: &str) -> Vec<String> {
        s.split_whitespace().map(|t| t.to_lowercase()).collect()
    }

    #[test]
    fn single_token_matches() {
        assert!(token_matches(&tokens("email"), &["Send an email"]));
    }

    #[test]
    fn single_token_no_match() {
        assert!(!token_matches(&tokens("calendar"), &["Send an email"]));
    }

    #[test]
    fn multi_token_all_present() {
        assert!(token_matches(
            &tokens("send email"),
            &["Send an email to recipients"]
        ));
    }

    #[test]
    fn multi_token_partial_fails() {
        // "upload" is present but "calendar" is not
        assert!(!token_matches(
            &tokens("upload calendar"),
            &["Upload a file with automatic metadata"]
        ));
    }

    #[test]
    fn case_insensitive() {
        assert!(token_matches(&tokens("GMAIL"), &["gws-gmail", "Gmail API"]));
    }

    #[test]
    fn match_across_fields() {
        // "drive" from the service name field, "upload" from the description field
        assert!(token_matches(
            &tokens("drive upload"),
            &["gws-drive-upload", "Upload a file with automatic metadata"]
        ));
    }

    #[test]
    fn empty_tokens_always_match() {
        assert!(token_matches(&[], &["anything here"]));
    }
}
