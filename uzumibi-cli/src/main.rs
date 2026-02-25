use clap::{Parser, Subcommand};
use include_dir::{Dir, include_dir};
use std::fs;
use std::io::Write;
use std::path::Path;

static TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

#[derive(Parser)]
#[command(name = "uzumibi")]
#[command(about = "Uzumibi CLI - Create a new edge application project powered by Ruby", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new edge application project
    New {
        /// Template type (cloudflare, cloudrun, fastly, spin, serviceworker, webworker)
        #[arg(short, long)]
        template: String,

        /// Project name, which will be used as the directory name
        project_name: String,

        /// Destination directory (defaults to project_name)
        #[arg(short, long)]
        dest_dir: Option<String>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New {
            template,
            project_name,
            dest_dir,
        } => {
            let dest = dest_dir.as_deref().unwrap_or(&project_name);
            create_project(&template, &project_name, dest)?;
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

fn create_project(
    template: &str,
    project_name: &str,
    dest_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if template exists
    let template_dir = TEMPLATES.get_dir(template).ok_or_else(|| {
        eprintln!("Available templates: {:?}", available_templates());
        format!("Template '{}' not found", template,)
    })?;

    // Check if target directory already exists
    let target_path = Path::new(dest_dir);
    if target_path.exists() {
        return Err(format!("Directory '{}' already exists", dest_dir).into());
    }

    // Create target directory
    fs::create_dir_all(target_path)?;

    println!("Creating project '{}'...", project_name);

    // Copy template files recursively
    copy_dir_recursive(template_dir, target_path, project_name, dest_dir, Path::new(""))?;

    println!(
        "\n✓ Successfully created project from template '{}'",
        template
    );
    println!("  Run 'cd {}' to get started!", dest_dir);
    print_project_next_steps(template, project_name);

    Ok(())
}

fn copy_dir_recursive(
    source: &Dir,
    target: &Path,
    project_name: &str,
    dest_dir: &str,
    relative_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Copy all files in current directory
    for file in source.files() {
        let file_name = file.path().file_name().unwrap();
        let file_name_str = file_name.to_str().unwrap();

        // Handle special case: Cargo.toml_ -> Cargo.toml
        let actual_file_name = if file_name_str == "Cargo.toml_" {
            "Cargo.toml"
        } else {
            file_name_str
        };

        let target_file = target.join(actual_file_name);
        let display_path = relative_path.join(actual_file_name);

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

        println!(
            "  \x1b[1mgenerate\x1b[0m {}/{}",
            dest_dir,
            display_path.display()
        );
    }

    // Recursively copy subdirectories
    for dir in source.dirs() {
        let dir_name = dir.path().file_name().unwrap();
        let target_subdir = target.join(dir_name);
        let new_relative_path = relative_path.join(dir_name);

        fs::create_dir_all(&target_subdir)?;
        copy_dir_recursive(dir, &target_subdir, project_name, dest_dir, &new_relative_path)?;
    }

    Ok(())
}

fn substitute_project_name(content: &str, project_name: &str) -> String {
    let project_name_underscore = project_name.replace('-', "_");

    content
        .replace("$$PROJECT_NAME$$", project_name)
        .replace("$$PROJECT_NAME_UNDERSCORE$$", &project_name_underscore)
}

fn print_project_next_steps(template: &str, project_name: &str) {
    println!("\nNext steps:");
    match template {
        "cloudflare" => {
            println!("  0. Install required tools (if not installed):");
            println!("     • Rust & Cargo:");
            println!(
                "     \x1b[36mcurl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh\x1b[0m"
            );
            println!(
                "     \x1b[36mrustup target add wasm32-unknown-unknown\x1b[0m"
            );
            println!("     • Node.js tools:");
            println!("     \x1b[36mnpm install -g pnpm wrangler\x1b[0m");
            println!();
            println!("  1. Install dependencies:");
            println!("     \x1b[36mpnpm install\x1b[0m");
            println!("  2. Build and start development server:");
            println!("     \x1b[36mpnpm run dev\x1b[0m");
            println!("  3. Deploy to Cloudflare:");
            println!("     \x1b[36mpnpm run deploy\x1b[0m");
        }
        "cloudrun" => {
            println!("  0. Install required tools and setup account:");
            println!("     • Docker:");
            println!("     Visit: https://docs.docker.com/get-docker/");
            println!("     • Google Cloud SDK:");
            println!("     Visit: https://cloud.google.com/sdk/docs/install");
            println!("     • Grant IAM role (if required):");
            println!(
                "     Set \x1b[33mCloud Run Developer\x1b[0m role to the default service account"
            );
            println!("     Visit: https://cloud.google.com/run/docs/securing/service-identity");
            println!();
            println!("  1. Build the project:");
            println!("     \x1b[36mdocker build -t {} .\x1b[0m", project_name);
            println!("  2. Test locally (optional):");
            println!(
                "     \x1b[36mdocker run -p 8080:8080 {}\x1b[0m",
                project_name
            );
            println!("  3. Deploy to Cloud Run:");
            println!("     \x1b[36mgcloud run deploy --source . --platform managed\x1b[0m");
        }
        "fastly" => {
            println!("  0. Install required tools (if not installed):");
            println!("     • Rust & Cargo:");
            println!(
                "     \x1b[36mcurl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh\x1b[0m"
            );
            println!(
                "     \x1b[36mrustup target add wasm32-wasip1\x1b[0m"
            );
            println!("     • Fastly CLI:");
            println!("     \x1b[36mbrew install fastly/tap/fastly\x1b[0m");
            println!("     Or visit: https://www.fastly.com/documentation/reference/tools/cli/");
            println!();
            println!("  1. Build the project:");
            println!("     \x1b[36mfastly compute build\x1b[0m");
            println!("  2. Start local development server:");
            println!("     \x1b[36mfastly compute serve\x1b[0m");
            println!("  3. Deploy to Fastly:");
            println!("     \x1b[36mfastly compute deploy\x1b[0m");
        }
        "spin" => {
            println!("  0. Install required tools (if not installed):");
            println!("     • Rust & Cargo:");
            println!(
                "     \x1b[36mcurl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh\x1b[0m"
            );
            println!(
                "     \x1b[36mrustup target add wasm32-wasip1\x1b[0m"
            );
            println!("     • Spin CLI:");
            println!(
                "     \x1b[36mcurl -fsSL https://developer.fermyon.com/downloads/install.sh | bash\x1b[0m"
            );
            println!("     Or visit: https://developer.fermyon.com/spin/install");
            println!();
            println!("  1. Build and start development server:");
            println!("     \x1b[36mspin build --up\x1b[0m");
            println!("  2. Or just start the server:");
            println!("     \x1b[36mspin up\x1b[0m");
            println!("  3. Deploy to Fermyon Cloud:");
            println!("     \x1b[36mspin deploy\x1b[0m");
        }
        "serviceworker" | "webworker" => {
            println!("  0. Install required tools (if not installed):");
            println!("     • Rust & Cargo:");
            println!(
                "     \x1b[36mcurl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh\x1b[0m"
            );
            println!(
                "     \x1b[36mrustup target add wasm32-unknown-unknown\x1b[0m"
            );
            println!();
            println!("  1. Build WebAssembly:");
            println!("     \x1b[36mmake wasm\x1b[0m");
            println!("  2. Start local server:");
            println!("     \x1b[36mmake serve\x1b[0m");
        }
        _ => {
            unreachable!("  Unknown template: {}", template);
        }
    }

    println!();
    match template {
        "serviceworker" | "webworker" => {
            println!(
                "  • After trying to bootstrap, edit \x1b[33mlib/app.rb\x1b[0m and \x1b[33mpublic/index.html\x1b[0m to develop your custom SPA application"
            );
        }
        _ => {
            println!(
                "  • After trying to bootstrap, edit \x1b[33mlib/app.rb\x1b[0m to develop your custom application"
            );
        }
    }
}
