use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "wctx",
    about = "WebContextTool - Manage Web project context (architecture, constraints, security) using Git branches",
    version,
    author
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Path to the Git repository
    #[arg(short, long, global = true, default_value = ".")]
    pub repo: String,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize the Web context repository structure
    Init,

    /// Registry commands for managing context structure
    #[command(subcommand)]
    Registry(RegistryCommands),

    /// Show context details by path (JSON format)
    Show(ShowArgs),

    /// Update context information
    Update(UpdateArgs),

    /// Scan a project and auto-generate context
    Scan(ScanArgs),

    /// Generate prompt context (JSON format)
    Generate(GenerateArgs),

    /// Configure wctx settings
    Config {
        /// Set repository path
        #[arg(short, long)]
        repo: Option<String>,
        /// Set user name for commits
        #[arg(short, long)]
        name: Option<String>,
        /// Set user email for commits
        #[arg(short, long)]
        email: Option<String>,
        /// Set language (zh/en)
        #[arg(short, long)]
        lang: Option<String>,
        /// Show current configuration
        #[arg(long)]
        show: bool,
        /// Reset to default configuration
        #[arg(long)]
        reset: bool,
    },

    /// Mount an existing WebContext repository
    Mount {
        #[arg(value_name = "PATH")]
        path: String,
    },

    /// Check if an existing repository meets wctx requirements
    Check {
        #[arg(value_name = "PATH")]
        path: Option<String>,
    },
}

/// Registry commands
#[derive(Subcommand)]
pub enum RegistryCommands {
    /// Create a new version (auto-increment from latest)
    New {
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Create a new category (architecture/constraint/security)
    Category {
        /// Category name (architecture/constraint/security)
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Create a tech-stack entry
    TechStack {
        /// Tech stack name (e.g., rust-actix, node-express)
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Create a module entry
    Module {
        /// Module path (e.g., users/auth)
        #[arg(value_name = "PATH")]
        path: String,
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Create a validation rule entry
    Validation {
        /// Validation name
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Create a permission entry
    Permission {
        /// Permission name
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Create an authentication entry
    Auth {
        /// Auth name (e.g., jwt, oauth)
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Create a CORS policy entry
    Cors {
        /// CORS policy name
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        description: Option<String>,
    },
}

#[derive(Args)]
pub struct ShowArgs {
    /// Context path (e.g., v1/architecture/tech-stack)
    #[arg(value_name = "PATH")]
    pub path: String,
}

#[derive(Args)]
pub struct UpdateArgs {
    /// Context path
    #[arg(value_name = "PATH")]
    pub path: String,
    /// Update in key:content format
    #[arg(value_name = "KEY:CONTENT")]
    pub update: String,
}

#[derive(Args)]
pub struct ScanArgs {
    /// Path to scan (default: current directory)
    #[arg(value_name = "PATH")]
    pub path: Option<String>,

    /// Output to a context entry
    #[arg(short, long)]
    pub output: Option<String>,
}

#[derive(Args)]
pub struct GenerateArgs {
    /// Version to generate context for (default: latest)
    #[arg(value_name = "VERSION")]
    pub version: Option<String>,

    /// Output format: json or prompt
    #[arg(short, long, default_value = "json")]
    pub format: String,
}
