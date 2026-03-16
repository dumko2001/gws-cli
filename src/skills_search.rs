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
use crate::registry::{PersonaRegistry, RecipeRegistry, PERSONAS_YAML, RECIPES_YAML};
use crate::services;

/// Entry point for `gws skills search <query>`.
pub async fn handle_skills_command(args: &[String]) -> Result<(), GwsError> {
    if args.is_empty() || args[0] != "search" {
        return Err(GwsError::Validation(
            "Usage: gws skills search <query>".to_string(),
        ));
    }

    let query = args
        .get(1)
        .ok_or_else(|| {
            GwsError::Validation(
                "No search query provided. Usage: gws skills search <query>".to_string(),
            )
        })?
        .to_lowercase();

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
