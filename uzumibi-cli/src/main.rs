use clap::{Parser, Subcommand};
use include_dir::{Dir, include_dir};
use std::fs;
use std::io::Write;
use std::path::Path;

static TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

#[derive(Parser)]
#[command(name = "uzumibi")]
#[command(about = "Uzumibi CLI - Create a new edge application project powered by Ruby", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new edge application project
    New {
        /// Template type (cloudflare, fastly, spin)
        #[arg(short, long)]
        template: String,

        /// Project name, which will be used as the directory name
        project_name: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New {
            template,
            project_name,
        } => {
            create_project(&template, &project_name)?;
        }
    }

    Ok(())
}

fn available_templates() -> Vec<&'static str> {
    TEMPLATES
        .dirs()
        .map(|dir| dir.path().file_name().unwrap().to_str().unwrap())
        .collect()
}

fn create_project(template: &str, project_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Check if template exists
    let template_dir = TEMPLATES.get_dir(template).ok_or_else(|| {
        eprintln!("Available templates: {:?}", available_templates());
        format!("Template '{}' not found", template,)
    })?;

    // Check if target directory already exists
    let target_path = Path::new(project_name);
    if target_path.exists() {
        return Err(format!("Directory '{}' already exists", project_name).into());
    }

    // Create target directory
    fs::create_dir_all(target_path)?;

    println!("Creating project '{}'...", project_name);

    // Copy template files recursively
    copy_dir_recursive(template_dir, target_path, project_name, Path::new(""))?;

    println!(
        "\n✓ Successfully created project from template '{}'",
        template
    );
    println!("  Run 'cd {}' to get started!", project_name);
    print_project_next_steps(template);

    Ok(())
}

fn copy_dir_recursive(
    source: &Dir,
    target: &Path,
    project_name: &str,
    relative_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Copy all files in current directory
    for file in source.files() {
        let file_name = file.path().file_name().unwrap();
        let target_file = target.join(file_name);
        let display_path = relative_path.join(file_name);

        let content = file.contents();
        let content_str = std::str::from_utf8(content);

        match content_str {
            Ok(text) => {
                // Text file - apply template substitution
                let substituted = substitute_project_name(text, project_name);
                let mut f = fs::File::create(&target_file)?;
                f.write_all(substituted.as_bytes())?;
            }
            Err(_) => {
                // Binary file - copy as-is
                fs::write(&target_file, content)?;
            }
        }

        println!("  \x1b[1mgenerate\x1b[0m {}", display_path.display());
    }

    // Recursively copy subdirectories
    for dir in source.dirs() {
        let dir_name = dir.path().file_name().unwrap();
        let target_subdir = target.join(dir_name);
        let new_relative_path = relative_path.join(dir_name);

        fs::create_dir_all(&target_subdir)?;
        copy_dir_recursive(dir, &target_subdir, project_name, &new_relative_path)?;
    }

    Ok(())
}

fn substitute_project_name(content: &str, project_name: &str) -> String {
    let project_name_underscore = project_name.replace('-', "_");

    content
        .replace("$$PROJECT_NAME$$", project_name)
        .replace("$$PROJECT_NAME_UNDERSCORE$$", &project_name_underscore)
}

fn print_project_next_steps(template: &str) {
    println!("\nNext steps:");

    match template {
        "cloudflare" => {
            println!("  1. Install dependencies:");
            println!("     \x1b[36mpnpm install\x1b[0m");
            println!("  2. Build and start development server:");
            println!("     \x1b[36mpnpm run dev\x1b[0m");
            println!("  3. Deploy to Cloudflare:");
            println!("     \x1b[36mpnpm run deploy\x1b[0m");
        }
        "fastly" => {
            println!("  1. Build the project:");
            println!("     \x1b[36mfastly compute build\x1b[0m");
            println!("  2. Start local development server:");
            println!("     \x1b[36mfastly compute serve\x1b[0m");
            println!("  3. Deploy to Fastly:");
            println!("     \x1b[36mfastly compute deploy\x1b[0m");
        }
        "spin" => {
            println!("  1. Build and start development server:");
            println!("     \x1b[36mspin build --up\x1b[0m");
            println!("  2. Or just start the server:");
            println!("     \x1b[36mspin up\x1b[0m");
            println!("  3. Deploy to Fermyon Cloud:");
            println!("     \x1b[36mspin deploy\x1b[0m");
        }
        _ => {
            println!("  See the project README for next steps.");
        }
    }
}
