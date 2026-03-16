use anyhow::{Context, Result, bail};
use colored::Colorize;
use std::fs;

use crate::git::GitRepo;
use crate::commands::registry::load_mapping;

/// Update context entry
pub fn execute(repo: &GitRepo, path: &str, update: &str) -> Result<()> {
    println!("{} Updating context: {}", "→".yellow(), path.cyan());
    println!();

    // Parse the update string (key:value)
    let parts: Vec<&str> = update.splitn(2, ':').collect();
    if parts.len() != 2 {
        bail!("Invalid update format. Use 'key:value'");
    }

    let key = parts[0].trim();
    let value = parts[1].trim();

    // Find the entry
    let mapping = load_mapping(repo)?;

    // Try exact match first
    let entry = if let Some(e) = mapping.get_by_path(path) {
        e.clone()
    } else {
        // Try with version prefix
        let latest = crate::commands::registry::get_latest_version(repo)?
            .context("No version found")?;
        let versioned_path = format!("{}/{}", latest, path);
        mapping.get_by_path(&versioned_path)
            .cloned()
            .context(format!("Path '{}' not found", path))?
    };

    // Checkout to the entry's branch
    repo.checkout(&entry.branch)?;

    // Read current content
    let content = fs::read_to_string("INFO.md")
        .or_else(|_| fs::read_to_string("ERROR.md"))
        .context("Failed to read content file")?;

    // Update the content
    let updated_content = update_markdown_field(&content, key, value)?;

    // Write updated content
    if std::path::Path::new("INFO.md").exists() {
        fs::write("INFO.md", &updated_content)?;
    } else {
        fs::write("ERROR.md", &updated_content)?;
    }

    // Commit the change
    repo.commit_files(
        &[std::path::Path::new(if std::path::Path::new("INFO.md").exists() { "INFO.md" } else { "ERROR.md" })],
        &format!("[UPDATE] {}: {}", key, value),
    )?;

    println!("{} Updated '{}' in {}", "✓".green().bold(), key, path.cyan());

    // Return to master
    repo.checkout("master")?;

    Ok(())
}

/// Update a markdown field
fn update_markdown_field(content: &str, key: &str, value: &str) -> Result<String> {
    let key_pattern = format!("## {}", key);
    let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    let mut found_key = false;
    let mut result_lines = Vec::new();
    let mut updated = false;

    for line in lines {
        if line.trim().to_lowercase() == key_pattern.to_lowercase() {
            found_key = true;
            result_lines.push(line);
        } else if found_key && line.starts_with("## ") {
            // Next section, add the new value before it
            result_lines.push(format!("{}", value));
            result_lines.push(String::new());
            result_lines.push(line);
            found_key = false;
            updated = true;
        } else if found_key && !line.trim().is_empty() && !line.starts_with('|') && !line.starts_with('-') {
            // This is the value line, replace it
            result_lines.push(format!("{}", value));
            found_key = false;
            updated = true;
        } else {
            result_lines.push(line);
        }
    }

    // If key was at the end
    if found_key && !updated {
        result_lines.push(format!("{}", value));
    }

    if !updated {
        // Try to add at the end
        result_lines.push(String::new());
        result_lines.push(key_pattern);
        result_lines.push(value.to_string());
    }

    Ok(result_lines.join("\n"))
}
