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

use crate::error::GwsError;
use crate::{commands, discovery};
use clap::Command;

/// Generates a bridge script for the given shell.
pub fn print_bridge_script(shell: &str) -> Result<(), GwsError> {
    match shell.to_lowercase().as_str() {
        "bash" => {
            println!(
                r#"
_gws_complete() {{
    local word="${{COMP_WORDS[COMP_CWORD]}}"
    local args=("${{COMP_WORDS[@]:1}}")
    
    local completions
    completions=$(gws __complete "${{args[@]}}")
    
    local filtered_completions=""
    while IFS= read -r line; do
        # Extract the completion part (everything before the first colon)
        filtered_completions+="${{line%%:*}} "
    done <<< "$completions"
    
    COMPREPLY=( $(compgen -W "$filtered_completions" -- "$word") )
}}
complete -F _gws_complete gws
"#
            );
        }
        "zsh" => {
            println!(
                r#"
#compdef gws

_gws() {{
    local -a completions
    local -a words_to_complete
    
    # Use the current words on the command line, skipping the first one ('gws')
    words_to_complete=("${{(@)words[2,$CURRENT]}}")
    
    # Get completions in 'name:description' format
    completions=("${{(@f)$(gws __complete "${{words_to_complete[@]}}")}}")
    
    if [[ -n "$completions" ]]; then
        _describe 'gws commands' completions
    fi
}}

compdef _gws gws
"#
            );
        }
        "fish" => {
            println!(
                r#"
function _gws_complete
    set -l cmd (commandline -opc)
    set -e cmd[1] # remove 'gws'
    
    # Fish expects 'name\tDescription'. Replace only the first colon.
    gws __complete $cmd | string replace -r '^([^:]+):(.*)' '$1\t$2'
end

complete -c gws -f -a "(_gws_complete)"
"#
            );
        }
        "powershell" => {
            println!(
                r#"
Register-ArgumentCompleter -NativeCommandName gws -ScriptBlock {{
    param($wordToComplete, $commandAst, $cursorPosition)

    # Get arguments from the AST, skipping the command itself ('gws').
    $arguments = $commandAst.CommandElements | Select-Object -Skip 1 | ForEach-Object {{ $_.Extent.Text }}

    # If the last character on the command line is a space, it means we are completing a new, empty argument.
    if ($commandAst.Extent.Text.EndsWith(" ")) {{
        $arguments += ""
    }}

    # Pass arguments to the completion command.
    gws __complete $arguments | ForEach-Object {{
        # Split only on the first colon to keep the rest of the description intact.
        $parts = $_.Split(":", 2)
        $name = $parts[0]
        $desc = if ($parts.Length -gt 1) {{ $parts[1] }} else {{ "" }}
        [System.Management.Automation.CompletionResult]::new($name, $name, 'ParameterValue', $desc)
    }}
}}
"#
            );
        }
        "elvish" => {
            println!(
                r#"
set edit:completion:arg-completer[gws] = {{ |@args|
    # remove 'gws'
    set args = $args[1..]
    gws __complete $@args | each {{ |line|
        local parts = [(str:split : $line &max=2)]
        local name = $parts[0]
        local desc = ''
        if (> (count $parts) 1) {{
            set desc = $parts[1]
        }}
        edit:complex-candidate $name &display=$name' ('$desc')'
    }}
}}
"#
            );
        }
        other => {
            return Err(GwsError::Validation(format!(
                "Dynamic completion bridge is not supported for '{}'. Supported: bash, zsh, fish, powershell, elvish",
                other
            )));
        }
    }
    Ok(())
}

/// Recursively find the subcommand matching the current arguments and print available next tokens.
pub fn handle_dynamic_completion(cli: &Command, args: &[String]) {
    let completions = get_completions(cli, args);
    for completion in completions {
        println!("{}", completion);
    }
}

fn get_completions(cli: &Command, args: &[String]) -> Vec<String> {
    let mut current_cmd = cli;
    let mut it = args.iter().peekable();
    let mut completions = Vec::new();

    // Skip 'gws' if present
    if let Some(&first) = it.peek() {
        if first == "gws" {
            it.next();
        }
    }

    let mut last_arg = None;
    let mut active_flag: Option<&clap::Arg> = None;

    // Walk the command tree using the provided args
    while let Some(arg) = it.next() {
        let is_last = it.peek().is_none();

        if arg.starts_with('-') && arg != "-" && arg != "--" {
            // This is a flag. We need to check if it takes a value and consume it
            // to avoid misinterpreting it as a subcommand.
            let arg_def = if arg.starts_with("--") {
                let name = arg.split('=').next().unwrap()[2..].to_string();
                current_cmd
                    .get_arguments()
                    .find(|a| a.get_long() == Some(&name))
            } else {
                // Short flag - handle combined flags like -vF
                let chars: Vec<char> = arg[1..].chars().collect();
                let mut found = None;
                for (i, &c) in chars.iter().enumerate() {
                    if let Some(def) = current_cmd
                        .get_arguments()
                        .find(|a| a.get_short() == Some(c))
                    {
                        if def.get_action().takes_values() {
                            // Only the last flag in a group can take a value
                            if i == chars.len() - 1 {
                                found = Some(def);
                            }
                            break;
                        }
                        found = Some(def);
                    }
                }
                found
            };

            if let Some(def) = arg_def {
                // If it's a flag that takes a value and isn't in `--key=value` form,
                // we need to consume the next argument as its value.
                if def.get_action().takes_values() && !arg.contains('=') {
                    if is_last {
                        // The flag is the last argument and it expects a value.
                        active_flag = Some(def);
                        last_arg = Some("");
                        break;
                    }
                    if let Some(val) = it.next() {
                        if it.peek().is_none() {
                            // This value is the last arg, so we are completing it!
                            active_flag = Some(def);
                            last_arg = Some(val.as_str());
                            break;
                        }
                    } else {
                        // The flag was the last arg, but it expects a value.
                        // We should complete for the value.
                        active_flag = Some(def);
                        last_arg = Some("");
                        break;
                    }
                }
            }
            if is_last {
                last_arg = Some(arg.as_str());
                break;
            }
            continue;
        }

        if is_last {
            // This is the last argument, we'll use it for filtering
            last_arg = Some(arg.as_str());
            break;
        }

        if let Some(subcmd) = current_cmd.find_subcommand(arg) {
            current_cmd = subcmd;
        } else {
            // Invalid subcommand, stop here
            break;
        }
    }

    let filter = last_arg.unwrap_or("");

    // If we are completing a flag value, try to suggest allowed values
    if let Some(flag) = active_flag {
        let val_parser = flag.get_value_parser();
        if let Some(pv) = val_parser.possible_values() {
            for val in pv {
                if val.get_name().starts_with(filter) {
                    completions.push(format!(
                        "{}:{}",
                        val.get_name(),
                        val.get_help().map(|h| h.to_string()).unwrap_or_default()
                    ));
                }
            }
        }
        return completions;
    }

    // Special case: if the last arg looks like a flag, complete flags
    if let Some(arg_filter) = filter.strip_prefix("--") {
        for arg in current_cmd.get_arguments() {
            if let Some(long) = arg.get_long() {
                if long.starts_with(arg_filter) {
                    let help = arg.get_help().map(|h| h.to_string()).unwrap_or_default();
                    completions.push(format!("--{}:{}", long, help));
                }
            }
        }
        return completions;
    } else if let Some(arg_filter) = filter.strip_prefix('-') {
        for arg in current_cmd.get_arguments() {
            if let Some(short) = arg.get_short() {
                if short.to_string().starts_with(arg_filter) {
                    let help = arg.get_help().map(|h| h.to_string()).unwrap_or_default();
                    completions.push(format!("-{}:{}", short, help));
                }
            }
        }
        return completions;
    }

    // Print subcommands with descriptions
    for subcmd in current_cmd.get_subcommands() {
        if !subcmd.is_hide_set() {
            let name = subcmd.get_name();
            if name.starts_with(filter) {
                let about = subcmd
                    .get_about()
                    .map(|a| a.to_string())
                    .unwrap_or_default();
                completions.push(format!("{}:{}", name, about));
            }
        }
    }
    completions
}

/// High-level handler for the `__complete` command.
pub async fn handle_complete_command(
    args: Vec<String>,
    static_cli_builder: fn() -> Command,
) -> Result<(), GwsError> {
    if args.is_empty() {
        let cmd = static_cli_builder();
        handle_dynamic_completion(&cmd, &[]);
        return Ok(());
    }

    let first_arg = &args[0];

    // Check if the first argument is a static subcommand (not a dynamic service)
    let is_service = crate::services::SERVICES
        .iter()
        .any(|s| s.aliases.contains(&first_arg.as_str()));
    if !is_service {
        let cmd = static_cli_builder();
        if cmd.find_subcommand(first_arg).is_some() {
            handle_dynamic_completion(&cmd, &args);
            return Ok(());
        }
    }

    match crate::parse_service_and_version(&args, first_arg) {
        Ok((api_name, version)) => {
            let doc_result = if api_name == "workflow" {
                Ok(discovery::RestDescription {
                    name: "workflow".to_string(),
                    description: Some("Cross-service productivity workflows".to_string()),
                    ..Default::default()
                })
            } else {
                discovery::fetch_discovery_document(&api_name, &version)
                    .await
                    .map_err(|e| GwsError::Discovery(format!("{e:#}")))
            };

            if let Ok(doc) = doc_result {
                let cmd = commands::build_cli(&doc);
                let sub_args = crate::filter_args_for_subcommand(&args, &api_name);
                handle_dynamic_completion(&cmd, &sub_args);
            }
        }
        Err(_) => {
            let cmd = static_cli_builder();
            handle_dynamic_completion(&cmd, &args);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{builder::PossibleValue, Arg, Command};

    fn test_cli() -> Command {
        Command::new("gws")
            .subcommand(
                Command::new("drive")
                    .about("Google Drive API")
                    .subcommand(
                        Command::new("files")
                            .about("Files resource")
                            .subcommand(Command::new("list").about("List files"))
                            .subcommand(Command::new("get").about("Get file metadata")),
                    )
                    .arg(
                        Arg::new("api-version")
                            .long("api-version")
                            .action(clap::ArgAction::Set),
                    )
                    .arg(Arg::new("format").long("format").value_parser([
                        PossibleValue::new("json").help("JSON format"),
                        PossibleValue::new("table").help("Table format"),
                    ])),
            )
            .subcommand(Command::new("auth").about("Authentication commands"))
    }

    #[test]
    fn test_get_completions_empty() {
        let cli = test_cli();
        let completions = get_completions(&cli, &[]);
        assert!(completions.contains(&"drive:Google Drive API".to_string()));
        assert!(completions.contains(&"auth:Authentication commands".to_string()));
    }

    #[test]
    fn test_get_completions_partial_subcommand() {
        let cli = test_cli();
        let completions = get_completions(&cli, &["dr".to_string()]);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0], "drive:Google Drive API");
    }

    #[test]
    fn test_get_completions_nested() {
        let cli = test_cli();
        let completions = get_completions(&cli, &["drive".to_string(), "f".to_string()]);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0], "files:Files resource");
    }

    #[test]
    fn test_get_completions_flags_with_values() {
        let cli = test_cli();
        // Completing after a flag that takes a value
        let completions = get_completions(
            &cli,
            &[
                "drive".to_string(),
                "--api-version".to_string(),
                "v3".to_string(),
                "f".to_string(),
            ],
        );
        assert_eq!(completions[0], "files:Files resource");
    }

    #[test]
    fn test_get_completions_flags_with_equals() {
        let cli = test_cli();
        let completions = get_completions(
            &cli,
            &[
                "drive".to_string(),
                "--api-version=v3".to_string(),
                "f".to_string(),
            ],
        );
        assert_eq!(completions[0], "files:Files resource");
    }

    #[test]
    fn test_get_completions_flag_value() {
        let cli = test_cli();
        // Input: gws drive --format j -> should suggest json
        let completions = get_completions(
            &cli,
            &["drive".to_string(), "--format".to_string(), "j".to_string()],
        );
        assert!(completions.iter().any(|c| c.starts_with("json:")));
    }

    #[test]
    fn test_get_completions_flag_last_arg() {
        let cli = test_cli();
        // Input: gws drive --format -> should suggest json, table
        let completions = get_completions(&cli, &["drive".to_string(), "--format".to_string()]);
        assert!(completions.iter().any(|c| c.starts_with("json:")));
        assert!(completions.iter().any(|c| c.starts_with("table:")));
    }

    #[test]
    fn test_get_completions_flag_names() {
        let cli = test_cli();
        let completions = get_completions(&cli, &["drive".to_string(), "--f".to_string()]);
        assert!(completions.iter().any(|c| c.starts_with("--format:")));
    }
}
