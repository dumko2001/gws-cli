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

    let query = args[1..].join(" ").to_lowercase();

    println!("Searching for skills matching \"{}\"...\n", query);

    let mut results = 0;

    // Search Services
    for svc in services::SERVICES {
        if svc.api_name.to_lowercase().contains(&query)
            || svc.description.to_lowercase().contains(&query)
            || svc
                .aliases
                .iter()
                .any(|a| a.to_lowercase().contains(&query))
        {
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

                    if full_helper_name.to_lowercase().contains(&query)
                        || about_clean.to_lowercase().contains(&query)
                    {
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
        if p.name.to_lowercase().contains(&query)
            || p.title.to_lowercase().contains(&query)
            || p.description.to_lowercase().contains(&query)
        {
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
        if r.name.to_lowercase().contains(&query)
            || r.title.to_lowercase().contains(&query)
            || r.description.to_lowercase().contains(&query)
        {
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
