use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Project type detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectType {
    Rust,
    NodeJS,
    Python,
    Go,
    Java,
    DotNet,
    Unknown,
}

/// Tech stack information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TechStack {
    pub language: Option<String>,
    pub web_framework: Option<String>,
    pub database: Option<String>,
    pub orm: Option<String>,
    pub dependencies: Vec<String>,
    pub dev_dependencies: Vec<String>,
}

/// Directory structure information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DirectoryStructure {
    pub root_files: Vec<String>,
    pub src_dirs: Vec<String>,
    pub config_files: Vec<String>,
    pub test_dirs: Vec<String>,
}

/// Security configuration detected
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityConfig {
    pub env_example_exists: bool,
    pub has_cors_config: bool,
    pub has_auth_config: bool,
    pub has_https_config: bool,
}

/// Complete scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub project_type: ProjectType,
    pub project_name: Option<String>,
    pub tech_stack: TechStack,
    pub directory_structure: DirectoryStructure,
    pub security: SecurityConfig,
    pub scanned_at: String,
}

impl Default for ProjectType {
    fn default() -> Self {
        ProjectType::Unknown
    }
}

/// Scan a project directory and extract context information
pub fn scan_project(path: &str) -> Result<ScanResult> {
    let project_path = Path::new(path);

    if !project_path.exists() {
        anyhow::bail!("Path does not exist: {}", path);
    }

    let project_type = detect_project_type(project_path)?;
    let project_name = detect_project_name(project_path, &project_type);
    let tech_stack = extract_tech_stack(project_path, &project_type)?;
    let directory_structure = extract_directory_structure(project_path);
    let security = detect_security_config(project_path);

    Ok(ScanResult {
        project_type,
        project_name,
        tech_stack,
        directory_structure,
        security,
        scanned_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    })
}

/// Detect the project type based on config files
fn detect_project_type(path: &Path) -> Result<ProjectType> {
    // Check for Rust
    if path.join("Cargo.toml").exists() {
        return Ok(ProjectType::Rust);
    }

    // Check for Node.js
    if path.join("package.json").exists() {
        return Ok(ProjectType::NodeJS);
    }

    // Check for Python
    if path.join("requirements.txt").exists()
        || path.join("pyproject.toml").exists()
        || path.join("setup.py").exists()
    {
        return Ok(ProjectType::Python);
    }

    // Check for Go
    if path.join("go.mod").exists() {
        return Ok(ProjectType::Go);
    }

    // Check for Java (Maven or Gradle)
    if path.join("pom.xml").exists() || path.join("build.gradle").exists() {
        return Ok(ProjectType::Java);
    }

    // Check for .NET
    if path.join("*.csproj").exists() || path.join("*.sln").exists() {
        return Ok(ProjectType::DotNet);
    }

    Ok(ProjectType::Unknown)
}

/// Detect project name
fn detect_project_name(path: &Path, project_type: &ProjectType) -> Option<String> {
    match project_type {
        ProjectType::Rust => {
            if let Ok(c) = fs::read_to_string(path.join("Cargo.toml")) {
                if let Ok(v) = toml::from_str::<toml::Value>(&c) {
                    if let Some(pkg) = v.get("package") {
                        if let Some(name) = pkg.get("name") {
                            return name.as_str().map(|s| s.to_string());
                        }
                    }
                }
            }
            None
        }

        ProjectType::NodeJS => {
            if let Ok(c) = fs::read_to_string(path.join("package.json")) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&c) {
                    if let Some(name) = v.get("name") {
                        return name.as_str().map(|s| s.to_string());
                    }
                }
            }
            None
        }

        ProjectType::Python => {
            // Try pyproject.toml first
            if let Ok(c) = fs::read_to_string(path.join("pyproject.toml")) {
                if let Ok(v) = toml::from_str::<toml::Value>(&c) {
                    if let Some(name) = v.get("project").and_then(|p| p.get("name")) {
                        return name.as_str().map(|s| s.to_string());
                    }
                }
            }
            // Try setup.py
            if let Ok(c) = fs::read_to_string(path.join("setup.py")) {
                if let Some(pos) = c.find("name=") {
                    let rest = &c[pos + 5..];
                    if let Some(end) = rest.find(',').or_else(|| rest.find(')')) {
                        let name = &rest[..end].trim().trim_matches('"').trim_matches('\'');
                        return Some(name.to_string());
                    }
                }
            }
            None
        }

        ProjectType::Go => {
            if let Ok(c) = fs::read_to_string(path.join("go.mod")) {
                if let Some(l) = c.lines().find(|l| l.starts_with("module ")) {
                    return Some(l.trim_start_matches("module ").to_string());
                }
            }
            None
        }

        _ => path.file_name().and_then(|n| n.to_str()).map(|s| s.to_string()),
    }
}

/// Extract tech stack information
fn extract_tech_stack(path: &Path, project_type: &ProjectType) -> Result<TechStack> {
    let mut tech_stack = TechStack::default();

    match project_type {
        ProjectType::Rust => {
            tech_stack.language = Some("Rust".to_string());
            if let Ok(content) = fs::read_to_string(path.join("Cargo.toml")) {
                if let Ok(parsed) = toml::from_str::<toml::Value>(&content) {
                    // Detect web framework
                    if let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_table()) {
                        for (key, _) in deps {
                            if key.contains("actix") || key.contains("axum") || key.contains("warp") || key.contains("rocket") {
                                tech_stack.web_framework = Some(key.clone());
                            }
                            if key.contains("diesel") || key.contains("sqlx") || key.contains("rusqlite") {
                                tech_stack.orm = Some(key.clone());
                            }
                            if key.contains("postgres") || key.contains("mysql") || key.contains("sqlite") {
                                tech_stack.database = Some(key.clone());
                            }
                            tech_stack.dependencies.push(key.clone());
                        }
                    }
                    if let Some(dev_deps) = parsed.get("dev-dependencies").and_then(|d| d.as_table()) {
                        for (key, _) in dev_deps {
                            tech_stack.dev_dependencies.push(key.clone());
                        }
                    }
                }
            }
        }

        ProjectType::NodeJS => {
            tech_stack.language = Some("JavaScript/TypeScript".to_string());
            if let Ok(content) = fs::read_to_string(path.join("package.json")) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                    // Detect dependencies
                    if let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_object()) {
                        for (key, _) in deps {
                            if key.contains("express") || key.contains("koa") || key.contains("fastify") || key.contains("nest") {
                                tech_stack.web_framework = Some(key.clone());
                            }
                            if key.contains("prisma") || key.contains("mongoose") || key.contains("sequelize") || key.contains("typeorm") {
                                tech_stack.orm = Some(key.clone());
                            }
                            if key.contains("postgres") || key.contains("mysql") || key.contains("mongodb") || key.contains("sqlite") {
                                tech_stack.database = Some(key.clone());
                            }
                            tech_stack.dependencies.push(key.clone());
                        }
                    }
                    if let Some(dev_deps) = parsed.get("devDependencies").and_then(|d| d.as_object()) {
                        for (key, _) in dev_deps {
                            tech_stack.dev_dependencies.push(key.clone());
                        }
                    }
                }
            }
        }

        ProjectType::Python => {
            tech_stack.language = Some("Python".to_string());
            // Try requirements.txt first
            if let Ok(content) = fs::read_to_string(path.join("requirements.txt")) {
                for line in content.lines() {
                    let dep = line.trim().split("==").next().unwrap_or(line.trim()).split(">=").next().unwrap_or(line.trim()).to_string();
                    if !dep.is_empty() && !dep.starts_with('#') {
                        if dep.contains("django") || dep.contains("flask") || dep.contains("fastapi") || dep.contains("aiohttp") {
                            tech_stack.web_framework = Some(dep.clone());
                        }
                        if dep.contains("sqlalchemy") || dep.contains("django-orm") || dep.contains("peewee") {
                            tech_stack.orm = Some(dep.clone());
                        }
                        tech_stack.dependencies.push(dep);
                    }
                }
            }
            // Also check pyproject.toml
            if let Ok(content) = fs::read_to_string(path.join("pyproject.toml")) {
                if let Ok(parsed) = toml::from_str::<toml::Value>(&content) {
                    if let Some(deps) = parsed.get("project").and_then(|p| p.get("dependencies")).and_then(|d| d.as_array()) {
                        for dep in deps {
                            if let Some(d) = dep.as_str() {
                                let name = d.split('[').next().unwrap_or(d).split("==").next().unwrap_or(d).to_string();
                                if name.contains("django") || name.contains("flask") || name.contains("fastapi") {
                                    tech_stack.web_framework = Some(name.clone());
                                }
                                tech_stack.dependencies.push(name);
                            }
                        }
                    }
                }
            }
        }

        ProjectType::Go => {
            tech_stack.language = Some("Go".to_string());
            if let Ok(content) = fs::read_to_string(path.join("go.mod")) {
                for line in content.lines() {
                    if line.starts_with("\t") || line.starts_with("    ") {
                        let dep = line.trim().split(' ').next().unwrap_or(line.trim()).to_string();
                        if !dep.is_empty() && !dep.starts_with("//") {
                            if dep.contains("gin") || dep.contains("echo") || dep.contains("chi") || dep.contains("fiber") {
                                tech_stack.web_framework = Some(dep.clone());
                            }
                            tech_stack.dependencies.push(dep);
                        }
                    }
                }
            }
        }

        _ => {}
    }

    Ok(tech_stack)
}

/// Extract directory structure
fn extract_directory_structure(path: &Path) -> DirectoryStructure {
    let mut structure = DirectoryStructure::default();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let path_type = entry.file_type().ok();

            if path_type.map(|t| t.is_dir()).unwrap_or(false) {
                // Check for common directories
                if name == "src" || name == "lib" || name == "app" || name == "source" {
                    structure.src_dirs.push(name);
                } else if name == "test" || name == "tests" || name == "spec" || name == "__tests__" {
                    structure.test_dirs.push(name);
                } else if name != "node_modules" && name != "target" && name != ".git" && name != "dist" && name != "build" {
                    // Other significant directories
                    structure.config_files.push(name);
                }
            } else if path_type.map(|t| t.is_file()).unwrap_or(false) {
                // Check for config files
                if name.ends_with(".json")
                    || name.ends_with(".toml")
                    || name.ends_with(".yaml")
                    || name.ends_with(".yml")
                    || name == ".env.example"
                    || name == ".gitignore"
                    || name == "Dockerfile"
                {
                    structure.config_files.push(name);
                } else {
                    structure.root_files.push(name);
                }
            }
        }
    }

    structure
}

/// Detect security configuration
fn detect_security_config(path: &Path) -> SecurityConfig {
    let mut security = SecurityConfig::default();

    // Check for .env.example
    security.env_example_exists = path.join(".env.example").exists();

    // Check for CORS config files
    security.has_cors_config = path.join("cors.json").exists()
        || path.join("cors.yaml").exists()
        || path.join("cors.toml").exists()
        || path.join("cors.config.js").exists();

    // Check for auth config
    security.has_auth_config = path.join("auth.config.js").exists()
        || path.join("auth.config.ts").exists()
        || path.join("auth.yaml").exists();

    // Check for HTTPS/ssl config
    security.has_https_config = path.join("ssl").exists()
        || path.join("certs").exists()
        || path.join(".https").exists();

    security
}

/// Output scan result as JSON
pub fn scan_to_json(result: &ScanResult) -> String {
    serde_json::to_string_pretty(result).unwrap_or_else(|_| "{}".to_string())
}

/// Generate architecture markdown from scan result
pub fn scan_to_architecture_markdown(result: &ScanResult) -> String {
    let mut md = String::new();

    md.push_str("# Architecture\n\n");
    md.push_str("## Type\narchitecture\n\n");
    md.push_str("## Tech Stack\n");

    if let Some(lang) = &result.tech_stack.language {
        md.push_str(&format!("- Language: {}\n", lang));
    }
    if let Some(fw) = &result.tech_stack.web_framework {
        md.push_str(&format!("- Web Framework: {}\n", fw));
    }
    if let Some(db) = &result.tech_stack.database {
        md.push_str(&format!("- Database: {}\n", db));
    }
    if let Some(orm) = &result.tech_stack.orm {
        md.push_str(&format!("- ORM: {}\n", orm));
    }

    if !result.tech_stack.dependencies.is_empty() {
        md.push_str("\n## Dependencies\n");
        for dep in &result.tech_stack.dependencies {
            md.push_str(&format!("- {}\n", dep));
        }
    }

    md.push_str("\n## Directory Structure\n```\n");

    for dir in &result.directory_structure.src_dirs {
        md.push_str(&format!("{}/\n", dir));
    }

    if !result.directory_structure.config_files.is_empty() {
        md.push_str("\nConfig:\n");
        for file in &result.directory_structure.config_files {
            md.push_str(&format!("  {}\n", file));
        }
    }

    md.push_str("```\n");

    md.push_str(&format!("\n## Scanned At\n{}\n", result.scanned_at));

    md
}
