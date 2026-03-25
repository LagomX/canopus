mod commands;
mod models;
mod store;

use clap::{Parser, Subcommand};
use colored::Colorize;
use commands::{analyze, attention, init, journal, observe, principles, reflect, sleep, status, task};

#[derive(Parser)]
#[command(
    name = "canopus",
    about = "Cognitive adversarial intelligence system — data entry CLI",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize ~/.canopus directory structure
    Init,
    /// Show today's data entry status
    Status,
    /// Record a journal entry
    Journal {
        /// Inline text (skips interactive prompt)
        #[arg(long)]
        text: Option<String>,
        /// Date to record for, YYYY-MM-DD (defaults to today)
        #[arg(long)]
        date: Option<String>,
        /// Mood score 1–10
        #[arg(long)]
        mood: Option<u8>,
        /// Energy score 1–10
        #[arg(long)]
        energy: Option<u8>,
    },
    /// Manage daily tasks
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },
    /// Record sleep data
    Sleep {
        /// Duration in hours
        hours: f64,
        /// Sleep quality score 1–5
        #[arg(long)]
        quality: u8,
        /// Bedtime as HH:MM
        #[arg(long)]
        bedtime: Option<String>,
        /// Wake time as HH:MM
        #[arg(long)]
        wake: Option<String>,
    },
    /// Record today's attention / screen time data
    Attention {
        /// Enter today's screen time interactively
        #[arg(long)]
        today: bool,
    },
    /// Generate raw observations from today's data
    Observe {
        /// Generate for a specific date (YYYY-MM-DD, defaults to today)
        #[arg(long)]
        date: Option<String>,
    },
    /// Reflect on past 7 days and find patterns
    Reflect,
    /// Manage principles (Ray Dalio state machine)
    Principles {
        #[command(subcommand)]
        action: commands::principles::PrinciplesAction,
    },
    /// Run cognitive adversary analysis via local Ollama
    Analyze {
        /// Force Level 3 (brutal) tone regardless of score
        #[arg(long)]
        brutal: bool,
        /// Analyze a specific past date (YYYY-MM-DD, defaults to today)
        #[arg(long)]
        date: Option<String>,
    },
}

#[derive(Subcommand)]
enum TaskAction {
    /// Add a new task for today
    Add {
        title: String,
        /// Priority: high, medium, low (default: medium)
        #[arg(long, default_value = "medium")]
        priority: String,
        /// Optional domain / category
        #[arg(long)]
        domain: Option<String>,
    },
    /// List today's tasks
    List,
    /// Mark a task done (1-based index or full id)
    Done { id_or_index: String },
    /// Skip a task (1-based index or full id)
    Skip {
        id_or_index: String,
        /// Optional reason for skipping
        reason: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init => init::run(),
        Commands::Status => status::run(),
        Commands::Journal { text, date, mood, energy } => journal::run(text, date, mood, energy),
        Commands::Task { action } => match action {
            TaskAction::Add { title, priority, domain } => task::run_add(title, priority, domain),
            TaskAction::List => task::run_list(),
            TaskAction::Done { id_or_index } => task::run_done(id_or_index),
            TaskAction::Skip { id_or_index, reason } => task::run_skip(id_or_index, reason),
        },
        Commands::Sleep { hours, quality, bedtime, wake } => {
            sleep::run(hours, quality, bedtime, wake)
        }
        Commands::Attention { today } => {
            if today {
                attention::run_today()
            } else {
                println!("Use {} to enter today's attention data.", "`canopus attention --today`".cyan());
                Ok(())
            }
        }
        Commands::Observe { date } => observe::run(date),
        Commands::Reflect => reflect::run(),
        Commands::Principles { action } => principles::run(action),
        Commands::Analyze { brutal, date } => analyze::run(brutal, date),
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}
