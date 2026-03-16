use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

use crate::git::GitRepo;
use crate::commands::registry::{MappingEntry, PathMapping, MAPPING_PATH};

/// Context entry for display
#[derive(Debug, Serialize, Deserialize)]
pub struct ContextEntry {
    pub path: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub content: String,
    pub branch: String,
}

/// Load mapping and get entry by path
fn get_entry_by_path(repo: &GitRepo, path: &str) -> Result<(PathMapping, MappingEntry)> {
    let mapping = load_mapping(repo)?;

    // Try exact match first
    if let Some(entry) = mapping.get_by_path(path).cloned() {
        return Ok((mapping, entry));
    }

    // Try with version prefix
    let latest = crate::commands::registry::get_latest_version(repo)?
        .context("No version found")?;

    let versioned_path = format!("{}/{}", latest, path);
    if let Some(entry) = mapping.get_by_path(&versioned_path).cloned() {
        return Ok((mapping, entry));
    }

    anyhow::bail!("Path '{}' not found", path)
}

fn load_mapping(repo: &GitRepo) -> Result<PathMapping> {
    let current_branch = repo.current_branch()?;
    if current_branch != "master" {
        repo.checkout("master")?;
    }

    let mapping = if let Ok(content) = fs::read_to_string(MAPPING_PATH) {
        serde_json::from_str(&content).unwrap_or_else(|_| PathMapping::new())
    } else {
        PathMapping::new()
    };

    if current_branch != "master" {
        repo.checkout(&current_branch)?;
    }

    Ok(mapping)
}

/// Display context entry
pub fn execute(repo: &GitRepo, path: &str) -> Result<()> {
    println!("{} Showing context for: {}", "→".yellow(), path.cyan());
    println!();

    let (_mapping, entry) = get_entry_by_path(repo, path)?;

    // Get content from the branch
    let content = if let Ok(Some(c)) = repo.get_file_from_branch(&entry.branch, "INFO.md") {
        c
    } else if let Ok(Some(c)) = repo.get_file_from_branch(&entry.branch, "ERROR.md") {
        c
    } else {
        // Try to get from current checkout if already on that branch
        let current = repo.current_branch()?;
        if current == entry.branch {
            fs::read_to_string("INFO.md")
                .or_else(|_| fs::read_to_string("ERROR.md"))
                .unwrap_or_else(|_| "No content found".to_string())
        } else {
            // Checkout to get the content
            if let Ok(_) = repo.checkout(&entry.branch) {
                fs::read_to_string("INFO.md")
                    .or_else(|_| fs::read_to_string("ERROR.md"))
                    .unwrap_or_else(|_| "No content found".to_string())
            } else {
                "Unable to retrieve content".to_string()
            }
        }
    };

    // Parse content and output as JSON
    let parsed_content = parse_markdown_to_json(&content);

    let context_entry = ContextEntry {
        path: entry.path,
        entry_type: entry.entry_type,
        content: parsed_content,
        branch: entry.branch,
    };

    println!("{}", serde_json::to_string_pretty(&context_entry).unwrap());

    // Return to master
    let _ = repo.checkout("master");

    Ok(())
}

/// Parse markdown content to structured JSON
fn parse_markdown_to_json(markdown: &str) -> String {
    let mut result = HashMap::new();
    let mut current_section = String::new();
    let mut section_content = Vec::new();

    for line in markdown.lines() {
        if line.starts_with("## ") {
            // Save previous section
            if !current_section.is_empty() && !section_content.is_empty() {
                result.insert(
                    current_section.trim().to_lowercase().replace(' ', "_"),
                    section_content.join("\n").trim().to_string(),
                );
            }
            current_section = line.trim_start_matches("## ").to_string();
            section_content = Vec::new();
        } else if line.starts_with("# ") {
            // Title section
            result.insert(
                "title".to_string(),
                line.trim_start_matches("# ").to_string(),
            );
        } else if !line.starts_with("```") && !line.trim().is_empty() {
            section_content.push(line.to_string());
        }
    }

    // Save last section
    if !current_section.is_empty() && !section_content.is_empty() {
        result.insert(
            current_section.trim().to_lowercase().replace(' ', "_"),
            section_content.join("\n").trim().to_string(),
        );
    }

    serde_json::to_string(&result).unwrap_or_else(|_| markdown.to_string())
}
