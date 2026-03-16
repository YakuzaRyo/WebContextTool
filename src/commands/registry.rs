use anyhow::{Context, Result, bail};
use chrono::Local;
use colored::Colorize;
use rand::Rng;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::git::GitRepo;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MappingEntry {
    pub path: String,
    pub branch: String,
    pub entry_type: String,
    pub parent: Option<String>,
    pub created: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct PathMapping {
    pub entries: HashMap<String, MappingEntry>,
    pub branches: HashMap<String, String>,
}

impl PathMapping {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            branches: HashMap::new(),
        }
    }

    pub fn add(&mut self, path: &str, branch: &str, entry_type: &str, parent: Option<&str>) {
        let entry = MappingEntry {
            path: path.to_string(),
            branch: branch.to_string(),
            entry_type: entry_type.to_string(),
            parent: parent.map(|s| s.to_string()),
            created: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };
        self.entries.insert(path.to_string(), entry);
        self.branches.insert(branch.to_string(), path.to_string());
    }

    pub fn get_by_path(&self, path: &str) -> Option<&MappingEntry> {
        self.entries.get(path)
    }
}

pub const MAPPING_PATH: &str = ".wctx/mapping.json";

fn generate_random_code() -> String {
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789".chars().collect();
    (0..8)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect()
}

pub fn load_mapping(repo: &GitRepo) -> Result<PathMapping> {
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

fn save_mapping(repo: &GitRepo, mapping: &PathMapping) -> Result<()> {
    let current_branch = repo.current_branch()?;

    if current_branch != "master" {
        repo.checkout("master")?;
    }

    fs::create_dir_all(".wctx")?;
    let content = serde_json::to_string_pretty(mapping)?;
    fs::write(MAPPING_PATH, content)?;
    repo.commit("[MAPPING] Update path mapping")?;

    if current_branch != "master" {
        repo.checkout(&current_branch)?;
    }

    Ok(())
}

pub fn get_latest_version(repo: &GitRepo) -> Result<Option<String>> {
    let branches = repo.list_branches()?;
    let version_regex = Regex::new(r"^v(\d+)$").unwrap();
    let mut max_version = 0;
    let mut latest_version = None;
    for (name, _) in branches {
        if let Some(caps) = version_regex.captures(&name) {
            let num: u32 = caps[1].parse().unwrap_or(0);
            if num > max_version {
                max_version = num;
                latest_version = Some(name);
            }
        }
    }
    Ok(latest_version)
}

/// Initialize the WebContext repository structure
pub fn init(repo: &GitRepo) -> Result<()> {
    println!("{}", "Initializing WebContextTool...".cyan().bold());

    if !repo.branch_exists("master")? {
        bail!("No master branch found. Please initialize git first.");
    }

    repo.checkout("master")?;

    // Create version tracking file
    let version_content = format!(
        r#"# Context Versions

## Current Version
None

## Version History

## Last Updated
{}
"#,
        Local::now().format("%Y-%m-%d %H:%M:%S")
    );
    fs::write("VERSION.md", version_content)?;
    repo.commit("[INIT] Create VERSION.md on master")?;
    println!("{} Updated master branch with VERSION.md", "✓".green());

    // Create architecture root branch
    if !repo.branch_exists("architecture")? {
        repo.checkout_new_branch_from("architecture", "master")?;
        fs::write(
            "INFO.md",
            "# Architecture Root\n\nThis is the root branch for all architecture contexts.\n",
        )?;
        repo.commit("[INIT] Create architecture root branch")?;
        println!("{} Created architecture root branch", "✓".green());
    }

    // Create constraint root branch
    if !repo.branch_exists("constraint")? {
        repo.checkout_new_branch_from("constraint", "master")?;
        fs::write(
            "INFO.md",
            "# Constraint Root\n\nThis is the root branch for all constraint contexts.\n",
        )?;
        repo.commit("[INIT] Create constraint root branch")?;
        println!("{} Created constraint root branch", "✓".green());
    }

    // Create security root branch
    if !repo.branch_exists("security")? {
        repo.checkout_new_branch_from("security", "master")?;
        fs::write(
            "INFO.md",
            "# Security Root\n\nThis is the root branch for all security contexts.\n",
        )?;
        repo.commit("[INIT] Create security root branch")?;
        println!("{} Created security root branch", "✓".green());
    }

    // Create mapping file
    repo.checkout("master")?;
    fs::create_dir_all(".wctx")?;
    let mapping = PathMapping::new();
    fs::write(MAPPING_PATH, serde_json::to_string_pretty(&mapping)?)?;
    repo.commit("[INIT] Create mapping file")?;
    println!("{} Created mapping file", "✓".green());

    println!("\n{}", "Initialization complete!".green().bold());
    println!("  Use 'wctx registry new' to create the first context version.");
    Ok(())
}

/// Create a new version
pub fn create_version(repo: &GitRepo, description: Option<&str>) -> Result<()> {
    let latest = get_latest_version(repo)?;
    let new_version_num = latest
        .as_ref()
        .map(|v| {
            Regex::new(r"v(\d+)")
                .unwrap()
                .captures(v)
                .and_then(|c| c[1].parse::<u32>().ok())
                .unwrap_or(0)
                + 1
        })
        .unwrap_or(1);

    let new_version = format!("v{}", new_version_num);
    let source = latest.unwrap_or_else(|| "architecture".to_string());

    println!(
        "{} Creating new version '{}' from '{}'...",
        "→".yellow(),
        new_version.cyan(),
        source.yellow()
    );

    let mut mapping = load_mapping(repo)?;

    // Create version branch from architecture root
    repo.checkout_new_branch_from(&new_version, "architecture")?;

    let date = Local::now().format("%Y-%m-%d %H:%M:%S");
    let desc = description.unwrap_or("New context version");

    fs::write(
        "INFO.md",
        format!(
            "# {}\n\n## Description\n{}\n\n## Created\n{}\n\n## Categories\n- architecture\n- constraint\n- security\n\n## Last Updated\n{}\n",
            new_version, desc, date, date
        ),
    )?;

    mapping.add(&new_version, &new_version, "version", Some("architecture"));
    save_mapping(repo, &mapping)?;

    // Update VERSION.md on master
    repo.checkout("master")?;
    let version_md = fs::read_to_string("VERSION.md").unwrap_or_default();
    let updated = version_md
        .replace(
            "## Current Version\nNone",
            &format!("## Current Version\n{}", new_version),
        )
        .replace(
            "## Version History",
            &format!(
                "## Version History\n- {}: {}\n",
                new_version, desc
            ),
        );
    fs::write("VERSION.md", updated)?;
    repo.commit(&format!("[VERSION] Record {} in master", new_version))?;

    repo.checkout(&new_version)?;

    println!(
        "{} Created version branch: {}",
        "✓".green().bold(),
        new_version.cyan()
    );
    Ok(())
}

/// Create a category (architecture/constraint/security)
pub fn create_category(repo: &GitRepo, name: &str, description: Option<&str>) -> Result<()> {
    // Validate category name
    let valid_categories = ["architecture", "constraint", "security"];
    if !valid_categories.contains(&name) {
        bail!(
            "Invalid category '{}'. Must be one of: {}",
            name,
            valid_categories.join(", ")
        );
    }

    let latest = get_latest_version(repo)?
        .context("No version found. Create one with 'wctx registry new'")?;

    let branch_code = generate_random_code();
    let branch_name = format!("{}-{}", name, branch_code);

    println!(
        "{} Creating category '{}' in {}...",
        "→".yellow(),
        name.cyan(),
        latest.yellow()
    );

    let mut mapping = load_mapping(repo)?;

    repo.checkout_new_branch_from(&branch_name, name)?;

    let date = Local::now().format("%Y-%m-%d %H:%M:%S");
    let desc = description.unwrap_or("New category");

    let info_content = format!(
        "# {}\n\n## Type\ncategory\n\n## Path\n{}\n\n## Description\n{}\n\n## Version\n{}\n\n## Created\n{}\n\n## Last Updated\n{}\n",
        name, name, desc, latest, date, date
    );
    fs::write("INFO.md", &info_content)?;

    let path = format!("{}/{}", latest, name);
    mapping.add(&path, &branch_name, "category", Some(name));
    save_mapping(repo, &mapping)?;

    fs::write("INFO.md", &info_content)?;

    repo.commit_files(
        &[Path::new("INFO.md")],
        &format!("[CATEGORY] Create {}", name),
    )?;

    println!("{} Created category: {}", "✓".green().bold(), name.cyan());

    repo.checkout("master")?;
    Ok(())
}

/// Create a tech-stack entry
pub fn create_tech_stack(repo: &GitRepo, name: &str, description: Option<&str>) -> Result<()> {
    let latest = get_latest_version(repo)?
        .context("No version found. Create one with 'wctx registry new'")?;

    let branch_code = generate_random_code();
    let branch_name = format!("arch-{}", branch_code);

    println!(
        "{} Creating tech-stack '{}'...",
        "→".yellow(),
        name.cyan()
    );

    let mut mapping = load_mapping(repo)?;

    // Get architecture category branch
    let arch_path = format!("{}/architecture", latest);
    let parent_branch = mapping
        .get_by_path(&arch_path)
        .map(|e| e.branch.clone())
        .unwrap_or_else(|| "architecture".to_string());

    repo.checkout_new_branch_from(&branch_name, &parent_branch)?;

    let date = Local::now().format("%Y-%m-%d %H:%M:%S");
    let desc = description.unwrap_or("Tech stack configuration");

    let content = format!(
        r#"# Tech Stack: {}

## Type
tech-stack

## Name
{}

## Description
{}

## Version
{}

## Created
{}

## Tech Stack Details

### Language
TBD

### Web Framework
TBD

### Database
TBD

### Dependencies
- None yet

## Last Updated
{}
"#,
        name, name, desc, latest, date, date
    );
    fs::write("INFO.md", &content)?;

    let path = format!("{}/architecture/{}", latest, name);
    mapping.add(&path, &branch_name, "tech-stack", Some(&parent_branch));
    save_mapping(repo, &mapping)?;

    fs::write("INFO.md", &content)?;

    repo.commit_files(
        &[Path::new("INFO.md")],
        &format!("[TECH-STACK] Create {}", name),
    )?;

    println!("{} Created tech-stack: {}", "✓".green().bold(), name.cyan());

    repo.checkout("master")?;
    Ok(())
}

/// Create a module entry
pub fn create_module(repo: &GitRepo, path: &str, description: Option<&str>) -> Result<()> {
    let latest = get_latest_version(repo)?
        .context("No version found. Create one with 'wctx registry new'")?;

    let parts: Vec<&str> = path.split('/').collect();
    let module_name = parts.last().unwrap_or(&path);

    let branch_code = generate_random_code();
    let branch_name = format!("arch-{}", branch_code);

    println!(
        "{} Creating module '{}'...",
        "→".yellow(),
        path.cyan()
    );

    let mut mapping = load_mapping(repo)?;

    // Get architecture category branch
    let arch_path = format!("{}/architecture", latest);
    let parent_branch = mapping
        .get_by_path(&arch_path)
        .map(|e| e.branch.clone())
        .unwrap_or_else(|| "architecture".to_string());

    repo.checkout_new_branch_from(&branch_name, &parent_branch)?;

    let date = Local::now().format("%Y-%m-%d %H:%M:%S");
    let desc = description.unwrap_or("Module configuration");

    let content = format!(
        r#"# Module: {}

## Type
module

## Path
{}

## Description
{}

## Version
{}

## Created
{}

## Module Structure

### Files
- None yet

### Dependencies
- None yet

## API Endpoints
- None yet

## Last Updated
{}
"#,
        module_name, path, desc, latest, date, date
    );
    fs::write("INFO.md", &content)?;

    let full_path = format!("{}/architecture/{}", latest, path);
    mapping.add(&full_path, &branch_name, "module", Some(&parent_branch));
    save_mapping(repo, &mapping)?;

    fs::write("INFO.md", &content)?;

    repo.commit_files(
        &[Path::new("INFO.md")],
        &format!("[MODULE] Create {}", path),
    )?;

    println!("{} Created module: {}", "✓".green().bold(), path.cyan());

    repo.checkout("master")?;
    Ok(())
}

/// Create a validation rule entry
pub fn create_validation(repo: &GitRepo, name: &str, description: Option<&str>) -> Result<()> {
    let latest = get_latest_version(repo)?
        .context("No version found. Create one with 'wctx registry new'")?;

    let branch_code = generate_random_code();
    let branch_name = format!("constraint-{}", branch_code);

    println!(
        "{} Creating validation '{}'...",
        "→".yellow(),
        name.cyan()
    );

    let mut mapping = load_mapping(repo)?;

    // Get constraint category branch
    let constraint_path = format!("{}/constraint", latest);
    let parent_branch = mapping
        .get_by_path(&constraint_path)
        .map(|e| e.branch.clone())
        .unwrap_or_else(|| "constraint".to_string());

    repo.checkout_new_branch_from(&branch_name, &parent_branch)?;

    let date = Local::now().format("%Y-%m-%d %H:%M:%S");
    let desc = description.unwrap_or("Validation rule");

    let content = format!(
        r#"# Validation Rule: {}

## Type
validation

## Name
{}

## Description
{}

## Version
{}

## Created
{}

## Validation Rules

| Field | Rule | Error Message |
|-------|------|---------------|
| TBD | | |

## Last Updated
{}
"#,
        name, name, desc, latest, date, date
    );
    fs::write("INFO.md", &content)?;

    let path = format!("{}/constraint/{}", latest, name);
    mapping.add(&path, &branch_name, "validation", Some(&parent_branch));
    save_mapping(repo, &mapping)?;

    fs::write("INFO.md", &content)?;

    repo.commit_files(
        &[Path::new("INFO.md")],
        &format!("[VALIDATION] Create {}", name),
    )?;

    println!("{} Created validation: {}", "✓".green().bold(), name.cyan());

    repo.checkout("master")?;
    Ok(())
}

/// Create a permission entry
pub fn create_permission(repo: &GitRepo, name: &str, description: Option<&str>) -> Result<()> {
    let latest = get_latest_version(repo)?
        .context("No version found. Create one with 'wctx registry new'")?;

    let branch_code = generate_random_code();
    let branch_name = format!("constraint-{}", branch_code);

    println!(
        "{} Creating permission '{}'...",
        "→".yellow(),
        name.cyan()
    );

    let mut mapping = load_mapping(repo)?;

    // Get constraint category branch
    let constraint_path = format!("{}/constraint", latest);
    let parent_branch = mapping
        .get_by_path(&constraint_path)
        .map(|e| e.branch.clone())
        .unwrap_or_else(|| "constraint".to_string());

    repo.checkout_new_branch_from(&branch_name, &parent_branch)?;

    let date = Local::now().format("%Y-%m-%d %H:%M:%S");
    let desc = description.unwrap_or("Permission configuration");

    let content = format!(
        r#"# Permission: {}

## Type
permission

## Name
{}

## Description
{}

## Version
{}

## Created
{}

## Permission Matrix

| Resource | Admin | User | Guest |
|----------|-------|------|-------|
| TBD | | | |

## Roles
- None yet

## Last Updated
{}
"#,
        name, name, desc, latest, date, date
    );
    fs::write("INFO.md", &content)?;

    let path = format!("{}/constraint/{}", latest, name);
    mapping.add(&path, &branch_name, "permission", Some(&parent_branch));
    save_mapping(repo, &mapping)?;

    fs::write("INFO.md", &content)?;

    repo.commit_files(
        &[Path::new("INFO.md")],
        &format!("[PERMISSION] Create {}", name),
    )?;

    println!("{} Created permission: {}", "✓".green().bold(), name.cyan());

    repo.checkout("master")?;
    Ok(())
}

/// Create an authentication entry
pub fn create_auth(repo: &GitRepo, name: &str, description: Option<&str>) -> Result<()> {
    let latest = get_latest_version(repo)?
        .context("No version found. Create one with 'wctx registry new'")?;

    let branch_code = generate_random_code();
    let branch_name = format!("security-{}", branch_code);

    println!(
        "{} Creating authentication '{}'...",
        "→".yellow(),
        name.cyan()
    );

    let mut mapping = load_mapping(repo)?;

    // Get security category branch
    let security_path = format!("{}/security", latest);
    let parent_branch = mapping
        .get_by_path(&security_path)
        .map(|e| e.branch.clone())
        .unwrap_or_else(|| "security".to_string());

    repo.checkout_new_branch_from(&branch_name, &parent_branch)?;

    let date = Local::now().format("%Y-%m-%d %H:%M:%S");
    let desc = description.unwrap_or("Authentication configuration");

    let content = format!(
        r#"# Authentication: {}

## Type
auth

## Name
{}

## Description
{}

## Version
{}

## Created
{}

## Authentication Method
TBD

## Token Configuration
- Token Expiry:
- Refresh Token:

## Last Updated
{}
"#,
        name, name, desc, latest, date, date
    );
    fs::write("INFO.md", &content)?;

    let path = format!("{}/security/{}", latest, name);
    mapping.add(&path, &branch_name, "auth", Some(&parent_branch));
    save_mapping(repo, &mapping)?;

    fs::write("INFO.md", &content)?;

    repo.commit_files(
        &[Path::new("INFO.md")],
        &format!("[AUTH] Create {}", name),
    )?;

    println!("{} Created authentication: {}", "✓".green().bold(), name.cyan());

    repo.checkout("master")?;
    Ok(())
}

/// Create a CORS policy entry
pub fn create_cors(repo: &GitRepo, name: &str, description: Option<&str>) -> Result<()> {
    let latest = get_latest_version(repo)?
        .context("No version found. Create one with 'wctx registry new'")?;

    let branch_code = generate_random_code();
    let branch_name = format!("security-{}", branch_code);

    println!(
        "{} Creating CORS policy '{}'...",
        "→".yellow(),
        name.cyan()
    );

    let mut mapping = load_mapping(repo)?;

    // Get security category branch
    let security_path = format!("{}/security", latest);
    let parent_branch = mapping
        .get_by_path(&security_path)
        .map(|e| e.branch.clone())
        .unwrap_or_else(|| "security".to_string());

    repo.checkout_new_branch_from(&branch_name, &parent_branch)?;

    let date = Local::now().format("%Y-%m-%d %H:%M:%S");
    let desc = description.unwrap_or("CORS policy");

    let content = format!(
        r#"# CORS Policy: {}

## Type
cors

## Name
{}

## Description
{}

## Version
{}

## Created
{}

## CORS Configuration
- Allowed Origins:
- Allowed Methods:
- Allowed Headers:
- Exposed Headers:
- Max Age:

## Last Updated
{}
"#,
        name, name, desc, latest, date, date
    );
    fs::write("INFO.md", &content)?;

    let path = format!("{}/security/{}", latest, name);
    mapping.add(&path, &branch_name, "cors", Some(&parent_branch));
    save_mapping(repo, &mapping)?;

    fs::write("INFO.md", &content)?;

    repo.commit_files(
        &[Path::new("INFO.md")],
        &format!("[CORS] Create {}", name),
    )?;

    println!("{} Created CORS policy: {}", "✓".green().bold(), name.cyan());

    repo.checkout("master")?;
    Ok(())
}

/// Check if a repository meets wctx requirements
pub fn check_repo(_current_repo: &GitRepo, path: &str) -> Result<()> {
    println!("{} Checking repository: {}", "→".yellow(), path.cyan());
    println!();

    if !GitRepo::is_valid(path) {
        bail!("No valid Git repository found at: {}", path);
    }

    let target_repo = GitRepo::open(path)?;

    let mut issues = Vec::new();
    let mut passed = Vec::new();

    // Check for required branches
    let has_master = target_repo.branch_exists("master")?;
    let has_architecture = target_repo.branch_exists("architecture")?;
    let has_constraint = target_repo.branch_exists("constraint")?;
    let has_security = target_repo.branch_exists("security")?;

    println!("{}", "1. Required Branches:".bold());
    if has_master {
        println!("  {} master branch exists", "✓".green());
        passed.push("master");
    } else {
        println!("  {} master branch missing", "✗".red());
        issues.push("Missing master branch");
    }
    if has_architecture {
        println!("  {} architecture branch exists", "✓".green());
        passed.push("architecture");
    } else {
        println!("  {} architecture branch missing", "✗".red());
        issues.push("Missing architecture branch");
    }
    if has_constraint {
        println!("  {} constraint branch exists", "✓".green());
        passed.push("constraint");
    } else {
        println!("  {} constraint branch missing", "✗".red());
        issues.push("Missing constraint branch");
    }
    if has_security {
        println!("  {} security branch exists", "✓".green());
        passed.push("security");
    } else {
        println!("  {} security branch missing", "✗".red());
        issues.push("Missing security branch");
    }

    // Check mapping file
    println!();
    println!("{}", "2. Mapping File:".bold());
    let mapping_path = format!("{}/.wctx/mapping.json", path);
    if std::path::Path::new(&mapping_path).exists() {
        let content = fs::read_to_string(&mapping_path)?;
        let mapping: PathMapping =
            serde_json::from_str(&content).context("Failed to parse mapping.json")?;
        println!("  {} mapping.json exists", "✓".green());
        println!("  {} entries: {}", "→".yellow(), mapping.entries.len());
        passed.push("mapping.json");
    } else {
        println!("  {} mapping.json missing", "✗".red());
        issues.push("Missing .wctx/mapping.json");
    }

    // Check VERSION.md
    println!();
    println!("{}", "3. Version File:".bold());
    let version_path = format!("{}/VERSION.md", path);
    if std::path::Path::new(&version_path).exists() {
        println!("  {} VERSION.md exists", "✓".green());
        passed.push("VERSION.md");
    } else {
        println!("  {} VERSION.md missing", "✗".red());
        issues.push("Missing VERSION.md");
    }

    // Check version branches
    println!();
    println!("{}", "4. Context Versions:".bold());
    let branches = target_repo.list_branches()?;
    let version_regex = Regex::new(r"^v(\d+)$").unwrap();
    let versions: Vec<_> = branches
        .iter()
        .filter(|(name, _)| version_regex.is_match(name))
        .collect();

    if versions.is_empty() {
        println!("  {} No version branches found", "⚠".yellow());
        issues.push("No version branches (v1, v2, etc.)");
    } else {
        println!("  {} Found {} version(s)", "✓".green(), versions.len());
        for (name, _) in &versions {
            println!("    - {}", name.cyan());
        }
        passed.push("version branches");
    }

    // Summary
    println!();
    println!("{}", "=".repeat(50));
    println!();
    println!("{}", "Summary:".bold());
    println!("  {} Checks passed: {}", "✓".green(), passed.len());
    println!("  {} Issues found: {}", "✗".red().bold(), issues.len());
    println!();

    if issues.is_empty() {
        println!(
            "{} Repository is a valid WebContext repository!",
            "✓".green().bold()
        );
    } else {
        println!("{}", "Issues:".bold());
        for issue in &issues {
            println!("  {} {}", "✗".red(), issue);
        }
        println!();
        println!(
            "{} Run 'wctx -r {} init' to initialize the repository",
            "→".yellow(),
            path.cyan()
        );
    }

    Ok(())
}

/// Mount an existing WebContext repository
pub fn mount_repo(_current_repo: &GitRepo, path: &str) -> Result<()> {
    println!("{} Mounting repository from: {}", "→".yellow(), path.cyan());

    if !GitRepo::is_valid(path) {
        bail!("No valid Git repository found at: {}", path);
    }

    let target_repo = GitRepo::open(path)?;

    let has_master = target_repo.branch_exists("master")?;
    let has_architecture = target_repo.branch_exists("architecture")?;
    let has_constraint = target_repo.branch_exists("constraint")?;
    let has_security = target_repo.branch_exists("security")?;

    println!();
    println!("{}", "Repository Status:".cyan().bold());
    println!();

    println!("{}", "Branches:".bold());
    if has_master {
        println!("  {} master", "✓".green());
    } else {
        println!("  {} master (missing)", "✗".red());
    }
    if has_architecture {
        println!("  {} architecture", "✓".green());
    } else {
        println!("  {} architecture (missing)", "✗".red());
    }
    if has_constraint {
        println!("  {} constraint", "✓".green());
    } else {
        println!("  {} constraint (missing)", "✗".red());
    }
    if has_security {
        println!("  {} security", "✓".green());
    } else {
        println!("  {} security (missing)", "✗".red());
    }

    let is_wctx_repo = has_master && has_architecture && has_constraint && has_security;

    println!();

    if is_wctx_repo {
        let current_branch = target_repo.current_branch()?;
        if current_branch != "master" {
            target_repo.checkout("master")?;
        }

        let mapping_path = format!("{}/.wctx/mapping.json", path);
        if std::path::Path::new(&mapping_path).exists() {
            let content = fs::read_to_string(&mapping_path)?;
            let mapping: PathMapping = serde_json::from_str(&content)?;
            println!("{}", "Mapping:".bold());
            println!("  {} entries: {}", "→".yellow(), mapping.entries.len());
            println!(
                "  {} branches tracked: {}",
                "→".yellow(),
                mapping.branches.len()
            );
        }

        let branches = target_repo.list_branches()?;
        let version_regex = Regex::new(r"^v(\d+)$").unwrap();
        let versions: Vec<_> = branches
            .iter()
            .filter(|(name, _)| version_regex.is_match(name))
            .collect();

        if !versions.is_empty() {
            println!();
            println!("{}", "Context Versions:".bold());
            for (name, _) in versions {
                println!("  {} {}", "→".yellow(), name.cyan());
            }
        }

        if current_branch != "master" {
            target_repo.checkout(&current_branch)?;
        }

        println!();
        println!(
            "{} Successfully mounted WebContext repository!",
            "✓".green().bold()
        );
        println!(
            "  Use 'wctx -r {} <command>' to work with this repository",
            path.cyan()
        );
    } else {
        println!(
            "{} Not a valid WebContext repository. Run 'wctx -r {} init' to initialize.",
            "⚠".yellow().bold(),
            path.cyan()
        );
    }

    Ok(())
}
