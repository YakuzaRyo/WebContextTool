# WebContextTool

A CLI tool for managing Web project context (architecture, constraints, security) using Git branches.

## Overview

WebContextTool helps you collect and manage Web project architecture design, business constraints, and security policies. It uses Git branches to organize different aspects of your project context, making it easy to version and query your project metadata.

## Features

- **Architecture Management**: Track tech stack, modules, and directory structure
- **Constraint Tracking**: Document validation rules and business logic
- **Security Policies**: Manage authentication, authorization, and CORS policies
- **Auto-Scanning**: Automatically detect project type and dependencies
- **JSON Output**: Generate context in JSON format for AI prompts

## Installation

```bash
cargo install webcontext
```

Or build from source:

```bash
git clone https://github.com/yourusername/WebContextTool.git
cd WebContextTool
cargo build --release
```

## Quick Start

```bash
# Initialize a new WebContext repository
wctx init

# Create a new version
wctx registry new --description "First version"

# Create categories
wctx registry category architecture
wctx registry category constraint
wctx registry category security

# Create tech-stack entry
wctx registry tech-stack rust-actix --description "Rust with Actix-web"

# Create module entry
wctx registry module users/auth --description "User authentication module"

# Create validation rule
wctx registry validation email-format --description "Email validation rule"

# Create permission entry
wctx registry permission admin-only --description "Admin permissions"

# Create authentication entry
wctx registry auth jwt --description "JWT authentication"

# Create CORS policy
wctx registry cors default --description "Default CORS policy"

# Show context (JSON format)
wctx show v1/architecture/tech-stack

# Update context
wctx update v1/architecture/tech-stack "description: Updated description"

# Scan a project
wctx scan /path/to/project

# Generate prompt context
wctx generate

# Check repository
wctx check
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `wctx init` | Initialize the Web context repository structure |
| `wctx registry new` | Create a new version |
| `wctx registry category <name>` | Create a category (architecture/constraint/security) |
| `wctx registry tech-stack <name>` | Create a tech-stack entry |
| `wctx registry module <path>` | Create a module entry |
| `wctx registry validation <name>` | Create a validation rule |
| `wctx registry permission <name>` | Create a permission entry |
| `wctx registry auth <name>` | Create an authentication entry |
| `wctx registry cors <name>` | Create a CORS policy |
| `wctx show <path>` | Show context details (JSON format) |
| `wctx update <path> "key:value"` | Update context entry |
| `wctx scan <path>` | Scan and detect project info |
| `wctx generate [version]` | Generate prompt context |
| `wctx config` | Configure settings |
| `wctx mount <path>` | Mount an existing repository |
| `wctx check` | Check repository compliance |

## Git Branch Structure

```
master (mapping file)
├── architecture (architecture root)
│   └── v1 (version branch)
│       └── v1-xxxxxx (entries)
├── constraint (constraints root)
│   └── v1
│       └── v1-xxxxxx
└── security (security root)
    └── v1
        └── v1-xxxxxx
```

## Scanner Support

The scanner can detect:

- **Rust**: Cargo.toml, dependencies
- **Node.js**: package.json, dependencies
- **Python**: requirements.txt, pyproject.toml
- **Go**: go.mod
- **Java**: pom.xml, build.gradle
- **.NET**: *.csproj, *.sln

## Configuration

```bash
# Show current configuration
wctx config --show

# Set repository path
wctx config --repo /path/to/repo

# Set user name
wctx config --name "Your Name"

# Set email
wctx config --email "your@email.com"

# Set language (zh/en)
wctx config --lang zh

# Reset to defaults
wctx config --reset
```

## Output Formats

### JSON Context

```json
{
  "version": "v1",
  "architecture": {
    "tech-stack": "# Tech Stack: rust-actix\n\n..."
  },
  "constraints": {
    "validation": "# Validation Rule: ...\n\n..."
  },
  "security": {
    "auth": "# Authentication: jwt\n\n..."
  },
  "generatedAt": "2026-03-16 20:00:00"
}
```

## License

MIT
