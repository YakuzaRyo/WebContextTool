use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

use crate::git::GitRepo;
use crate::commands::registry::{load_mapping, get_latest_version};

/// Complete context output for prompts
#[derive(Debug, Serialize, Deserialize)]
pub struct ContextOutput {
    pub version: String,
    pub architecture: Option<HashMap<String, String>>,
    pub constraints: Option<HashMap<String, String>>,
    pub security: Option<HashMap<String, String>>,
    #[serde(rename = "generatedAt")]
    pub generated_at: String,
}

/// Generate context for a version
pub fn execute(repo: &GitRepo, version: Option<String>, format: &str) -> Result<()> {
    let version_str = match version {
        Some(v) => v,
        None => {
            let latest = get_latest_version(repo)?;
            latest.context("No version found. Create one with 'wctx registry new'")?
        }
    };

    println!("{} Generating context for: {}", "→".yellow(), version_str.cyan());
    println!();

    let mapping = load_mapping(repo)?;

    // Collect all entries for this version
    let mut architecture: Option<HashMap<String, String>> = None;
    let mut constraints: Option<HashMap<String, String>> = None;
    let mut security: Option<HashMap<String, String>> = None;

    for (path, entry) in &mapping.entries {
        if !path.starts_with(&format!("{}/", version_str)) {
            continue;
        }

        let content = get_entry_content(repo, &entry.branch)?;
        let short_path = path.trim_start_matches(&format!("{}/", version_str));

        // Determine which category this belongs to
        if short_path.starts_with("architecture") {
            if architecture.is_none() {
                architecture = Some(HashMap::new());
            }
            if let Some(ref mut arch) = architecture {
                arch.insert(short_path.to_string(), content);
            }
        } else if short_path.starts_with("constraint") {
            if constraints.is_none() {
                constraints = Some(HashMap::new());
            }
            if let Some(ref mut cons) = constraints {
                cons.insert(short_path.to_string(), content);
            }
        } else if short_path.starts_with("security") {
            if security.is_none() {
                security = Some(HashMap::new());
            }
            if let Some(ref mut sec) = security {
                sec.insert(short_path.to_string(), content);
            }
        }
    }

    let output = ContextOutput {
        version: version_str.clone(),
        architecture,
        constraints,
        security,
        generated_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    match format {
        "prompt" => {
            let prompt = generate_prompt(&output);
            println!("{}", prompt);
        }
        _ => {
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
    }

    Ok(())
}

/// Get content from a branch
fn get_entry_content(repo: &GitRepo, branch: &str) -> Result<String> {
    // Try to get from current checkout
    let current = repo.current_branch()?;
    if current == branch {
        if let Ok(content) = fs::read_to_string("INFO.md") {
            return Ok(content);
        }
        if let Ok(content) = fs::read_to_string("ERROR.md") {
            return Ok(content);
        }
    }

    // Try to get from branch directly
    if let Ok(Some(content)) = repo.get_file_from_branch(branch, "INFO.md") {
        return Ok(content);
    }
    if let Ok(Some(content)) = repo.get_file_from_branch(branch, "ERROR.md") {
        return Ok(content);
    }

    // Checkout and read
    repo.checkout(branch)?;
    let content = fs::read_to_string("INFO.md")
        .or_else(|_| fs::read_to_string("ERROR.md"))
        .unwrap_or_else(|_| "No content".to_string());
    repo.checkout("master")?;

    Ok(content)
}

/// Generate a formatted prompt from context
fn generate_prompt(context: &ContextOutput) -> String {
    let mut prompt = String::new();

    prompt.push_str("# Web Application Context\n\n");
    prompt.push_str(&format!("## Version: {}\n\n", context.version));
    prompt.push_str(&format!("Generated: {}\n\n", context.generated_at));

    // Architecture section
    if let Some(ref arch) = context.architecture {
        if !arch.is_empty() {
            prompt.push_str("---\n\n");
            prompt.push_str("# Architecture\n\n");
            for (path, content) in arch {
                prompt.push_str(&format!("## {}\n\n", path));
                prompt.push_str(content);
                prompt.push_str("\n\n");
            }
        }
    }

    // Constraints section
    if let Some(ref cons) = context.constraints {
        if !cons.is_empty() {
            prompt.push_str("---\n\n");
            prompt.push_str("# Constraints\n\n");
            for (path, content) in cons {
                prompt.push_str(&format!("## {}\n\n", path));
                prompt.push_str(content);
                prompt.push_str("\n\n");
            }
        }
    }

    // Security section
    if let Some(ref sec) = context.security {
        if !sec.is_empty() {
            prompt.push_str("---\n\n");
            prompt.push_str("# Security\n\n");
            for (path, content) in sec {
                prompt.push_str(&format!("## {}\n\n", path));
                prompt.push_str(content);
                prompt.push_str("\n\n");
            }
        }
    }

    prompt
}
