use clap::{Parser, Subcommand};
use crate::db::DbConnection;
use crate::repo::ProjectRepo;
use anyhow::{Context, Result};

#[derive(Parser)]
#[command(name = "task")]
#[command(about = "Task Ninja - A powerful command-line task management tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Project management commands
    Projects {
        #[command(subcommand)]
        subcommand: ProjectCommands,
    },
}

#[derive(Subcommand)]
pub enum ProjectCommands {
    /// Create a new project
    Add {
        /// Project name (supports nested projects with dot notation, e.g., admin.email)
        name: String,
    },
    /// List projects
    List {
        /// Include archived projects
        #[arg(long)]
        archived: bool,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Rename a project
    Rename {
        /// Current project name
        old_name: String,
        /// New project name
        new_name: String,
        /// Force merge if new name already exists
        #[arg(long)]
        force: bool,
    },
    /// Archive a project
    Archive {
        /// Project name to archive
        name: String,
    },
    /// Unarchive a project
    Unarchive {
        /// Project name to unarchive
        name: String,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Projects { subcommand } => handle_projects(subcommand),
    }
}

fn handle_projects(cmd: ProjectCommands) -> Result<()> {
    let conn = DbConnection::connect()
        .context("Failed to connect to database")?;
    
    match cmd {
        ProjectCommands::Add { name } => {
            // Check if project already exists
            if let Some(_) = ProjectRepo::get_by_name(&conn, &name)? {
                eprintln!("Error: Project '{}' already exists", name);
                std::process::exit(1);
            }
            
            let project = ProjectRepo::create(&conn, &name)
                .context("Failed to create project")?;
            
            println!("Created project '{}' (id: {})", project.name, project.id.unwrap());
            Ok(())
        }
        ProjectCommands::List { archived, json } => {
            let projects = ProjectRepo::list(&conn, archived)
                .context("Failed to list projects")?;
            
            if json {
                let json_output = serde_json::to_string_pretty(&projects)
                    .context("Failed to serialize projects to JSON")?;
                println!("{}", json_output);
            } else {
                if projects.is_empty() {
                    println!("No projects found.");
                } else {
                    for project in projects {
                        let status = if project.is_archived { "[archived]" } else { "" };
                        println!("{} {}", project.name, status);
                    }
                }
            }
            Ok(())
        }
        ProjectCommands::Rename { old_name, new_name, force } => {
            // Check if old project exists
            if ProjectRepo::get_by_name(&conn, &old_name)?.is_none() {
                eprintln!("Error: Project '{}' not found", old_name);
                std::process::exit(1);
            }
            
            // Check if new name already exists
            if let Some(_) = ProjectRepo::get_by_name(&conn, &new_name)? {
                if force {
                    // Merge projects
                    ProjectRepo::merge(&conn, &old_name, &new_name)
                        .context("Failed to merge projects")?;
                    println!("Merged project '{}' into '{}'", old_name, new_name);
                } else {
                    eprintln!("Error: Project '{}' already exists. Use --force to merge.", new_name);
                    std::process::exit(1);
                }
            } else {
                // Simple rename
                ProjectRepo::rename(&conn, &old_name, &new_name)
                    .context("Failed to rename project")?;
                println!("Renamed project '{}' to '{}'", old_name, new_name);
            }
            Ok(())
        }
        ProjectCommands::Archive { name } => {
            ProjectRepo::archive(&conn, &name)
                .context("Failed to archive project")?;
            println!("Archived project '{}'", name);
            Ok(())
        }
        ProjectCommands::Unarchive { name } => {
            ProjectRepo::unarchive(&conn, &name)
                .context("Failed to unarchive project")?;
            println!("Unarchived project '{}'", name);
            Ok(())
        }
    }
}
