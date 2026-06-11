mod api;
mod display;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hn", about = "Hacker News CLI", version)]
struct Cli {
    /// Output as JSON
    #[arg(short, long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show top stories
    Top {
        /// Number of stories to show
        #[arg(short = 'n', long, default_value = "30")]
        limit: usize,
    },
    /// Show new stories
    New {
        #[arg(short = 'n', long, default_value = "30")]
        limit: usize,
    },
    /// Show best stories
    Best {
        #[arg(short = 'n', long, default_value = "30")]
        limit: usize,
    },
    /// Show Ask HN stories
    Ask {
        #[arg(short = 'n', long, default_value = "30")]
        limit: usize,
    },
    /// Show Show HN stories
    Show {
        #[arg(short = 'n', long, default_value = "30")]
        limit: usize,
    },
    /// Show job stories
    Jobs {
        #[arg(short = 'n', long, default_value = "30")]
        limit: usize,
    },
    /// View a story and its comments by ID
    Story {
        /// Story ID
        id: u64,
    },
    /// View a user profile
    User {
        /// Username
        id: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if cli.json {
        colored::control::set_override(false);
    }

    let client = api::Client::new();

    let result = match cli.command {
        Some(Commands::Top { limit }) => cmd_story_list(&client, "top", limit, cli.json).await,
        Some(Commands::New { limit }) => cmd_story_list(&client, "new", limit, cli.json).await,
        Some(Commands::Best { limit }) => cmd_story_list(&client, "best", limit, cli.json).await,
        Some(Commands::Ask { limit }) => cmd_story_list(&client, "ask", limit, cli.json).await,
        Some(Commands::Show { limit }) => cmd_story_list(&client, "show", limit, cli.json).await,
        Some(Commands::Jobs { limit }) => cmd_story_list(&client, "jobs", limit, cli.json).await,
        Some(Commands::Story { id }) => cmd_story(&client, id, cli.json).await,
        Some(Commands::User { id }) => cmd_user(&client, &id, cli.json).await,
        None => cmd_story_list(&client, "top", 30, cli.json).await,
    };

    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

async fn cmd_story_list(
    client: &api::Client,
    list_type: &str,
    limit: usize,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let ids = match list_type {
        "top" => client.get_top_stories().await?,
        "new" => client.get_new_stories().await?,
        "best" => client.get_best_stories().await?,
        "ask" => client.get_ask_stories().await?,
        "show" => client.get_show_stories().await?,
        "jobs" => client.get_job_stories().await?,
        _ => unreachable!("invalid list_type"),
    };

    let stories = client.get_stories_with_details(&ids, limit).await?;

    if json {
        display::print_story_list_json(&stories);
    } else {
        display::print_story_list(&stories, 1);
    }

    Ok(())
}

async fn cmd_story(
    client: &api::Client,
    id: u64,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let story = client.get_item(id).await?;
    let comments = client
        .get_all_comments(story.kids.as_deref().unwrap_or(&[]))
        .await?;

    if json {
        display::print_story_json(&story, &comments);
    } else {
        display::print_story_detail(&story, &comments, display::term_width());
    }

    Ok(())
}

async fn cmd_user(
    client: &api::Client,
    id: &str,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let user = client.get_user(id).await?;

    if json {
        display::print_user_json(&user);
    } else {
        display::print_user(&user);
    }

    Ok(())
}
