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

//! Registry definitions for Personas and Recipes.

use serde::Deserialize;

pub const PERSONAS_YAML: &str = include_str!("../registry/personas.yaml");
pub const RECIPES_YAML: &str = include_str!("../registry/recipes.yaml");

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
