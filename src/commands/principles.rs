use crate::models::principle::{Evidence, Principle, PrincipleStatus, StatusTransition, Validation};
use crate::store::{get_canopus_dir, get_today_str, is_initialized};
use clap::Subcommand;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

// ── Storage ───────────────────────────────────────────────────────────────────

fn get_principles_dir() -> PathBuf {
    get_canopus_dir().join("principles")
}

#[derive(Debug, Serialize, Deserialize)]
struct PrincipleIndexEntry {
    id: String,
    title: String,
    status: PrincipleStatus,
    domain: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PrincipleIndex {
    count: u32,
    principles: Vec<PrincipleIndexEntry>,
}

fn load_index() -> PrincipleIndex {
    let path = get_principles_dir().join("index.json");
    if !path.exists() {
        return PrincipleIndex { count: 0, principles: vec![] };
    }
    let content = fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&content).unwrap_or(PrincipleIndex { count: 0, principles: vec![] })
}

fn save_index(index: &PrincipleIndex) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_principles_dir().join("index.json");
    fs::write(&path, serde_json::to_string_pretty(index)?)?;
    Ok(())
}

fn principle_path(id: &str) -> PathBuf {
    get_principles_dir().join(format!("{}.json", id))
}

fn load_principle(id: &str) -> Option<Principle> {
    let content = fs::read_to_string(principle_path(id)).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_principle(p: &Principle) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(principle_path(&p.id), serde_json::to_string_pretty(p)?)?;
    Ok(())
}

fn resolve_id(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    if input.starts_with("principle_") {
        return Ok(input.to_string());
    }
    match input.parse::<u32>() {
        Ok(n) => Ok(format!("principle_{:03}", n)),
        Err(_) => Err(format!("Invalid principle id: '{}'", input).into()),
    }
}

fn next_id(index: &PrincipleIndex) -> String {
    let max_num = index
        .principles
        .iter()
        .filter_map(|p| p.id.strip_prefix("principle_"))
        .filter_map(|n| n.parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    format!("principle_{:03}", max_num + 1)
}

fn update_index_status(
    id: &str,
    status: PrincipleStatus,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut index = load_index();
    for entry in &mut index.principles {
        if entry.id == id {
            entry.status = status;
            break;
        }
    }
    save_index(&index)
}

// ── Interactive helpers ───────────────────────────────────────────────────────

fn read_line(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    input.trim().to_string()
}

fn read_required(prompt: &str) -> String {
    loop {
        let s = read_line(prompt);
        if !s.is_empty() {
            return s;
        }
        println!("{}", "This field is required.".yellow());
    }
}

fn read_optional(prompt: &str) -> Option<String> {
    let s = read_line(prompt);
    if s.is_empty() { None } else { Some(s) }
}

fn read_multiline(prompt: &str) -> String {
    println!("{}", prompt.cyan());
    let stdin = io::stdin();
    let mut lines: Vec<String> = Vec::new();
    let mut consecutive_empty = 0usize;
    for line in stdin.lock().lines() {
        let line = line.unwrap_or_default();
        if line.is_empty() {
            consecutive_empty += 1;
            if consecutive_empty >= 2 {
                break;
            }
            lines.push(line);
        } else {
            consecutive_empty = 0;
            lines.push(line);
        }
    }
    while lines.last().map_or(false, |l: &String| l.is_empty()) {
        lines.pop();
    }
    lines.join("\n")
}

fn confirm(prompt: &str) -> bool {
    print!("{} [y/N]: ", prompt);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    input.trim().eq_ignore_ascii_case("y")
}

fn sep() {
    println!("{}", "━".repeat(38));
}

fn status_colored(status: &PrincipleStatus) -> colored::ColoredString {
    match status {
        PrincipleStatus::Confirmed  => "confirmed".green().bold(),
        PrincipleStatus::Validated  => "validated".yellow().bold(),
        PrincipleStatus::Candidate  => "candidate".white(),
        PrincipleStatus::Deprecated => "deprecated".dimmed(),
    }
}

fn status_label(status: &PrincipleStatus) -> colored::ColoredString {
    match status {
        PrincipleStatus::Confirmed  => "CONFIRMED".green().bold(),
        PrincipleStatus::Validated  => "VALIDATED".yellow().bold(),
        PrincipleStatus::Candidate  => "CANDIDATE".white(),
        PrincipleStatus::Deprecated => "DEPRECATED".dimmed(),
    }
}

// ── Sub-command implementations ───────────────────────────────────────────────

pub fn run_list() -> Result<(), Box<dyn std::error::Error>> {
    if !is_initialized() {
        println!("{}", "Canopus not initialized. Run `canopus init` first.".red());
        return Ok(());
    }

    let index = load_index();
    sep();
    println!("{}", "PRINCIPLES".bold());
    sep();

    for status in &[
        PrincipleStatus::Confirmed,
        PrincipleStatus::Validated,
        PrincipleStatus::Candidate,
        PrincipleStatus::Deprecated,
    ] {
        let matching: Vec<&PrincipleIndexEntry> =
            index.principles.iter().filter(|p| &p.status == status).collect();

        println!("● {} ({})", status_label(status), matching.len());

        if matching.is_empty() {
            println!("  {}", "(none)".dimmed());
        } else {
            for entry in matching {
                let num = entry.id.trim_start_matches("principle_");
                let domain = entry.domain.as_deref().unwrap_or("?");
                println!("  #{} [{}] {}", num, domain, entry.title);
            }
        }
    }

    Ok(())
}

pub fn run_show(id_input: String) -> Result<(), Box<dyn std::error::Error>> {
    if !is_initialized() {
        println!("{}", "Canopus not initialized. Run `canopus init` first.".red());
        return Ok(());
    }

    let id = resolve_id(&id_input)?;
    let p = load_principle(&id)
        .ok_or_else(|| format!("Principle '{}' not found.", id_input))?;

    let num = p.id.trim_start_matches("principle_");
    let domain = p.domain.as_deref().unwrap_or("?");

    sep();
    println!("#{} · {} · {}", num, status_colored(&p.status), domain.cyan());
    sep();
    println!("{}", p.title.bold());
    println!("{}", p.description);
    println!();

    // Evidence
    let ev_note = if p.status == PrincipleStatus::Candidate && p.evidence.len() < 3 {
        format!("({}/{} needed to validate)", p.evidence.len(), 3)
    } else {
        String::new()
    };
    println!("{} {}", "EVIDENCE".bold(), ev_note.dimmed());
    if p.evidence.is_empty() {
        println!("  {}", "(none)".dimmed());
    } else {
        for e in &p.evidence {
            println!("  {}  {}", e.date.dimmed(), e.description);
            if let Some(oid) = &e.observation_id {
                println!("        obs: {}", oid.dimmed());
            }
        }
    }
    println!();

    // Validations
    println!("{}", "VALIDATIONS".bold());
    if p.validations.is_empty() {
        println!("  {}", "(none yet)".dimmed());
    } else {
        for v in &p.validations {
            println!("  {}  Decision: {}", v.date.dimmed(), v.decision);
            println!("        Outcome: {}", v.outcome);
        }
    }
    println!();

    // History
    println!("{}", "HISTORY".bold());
    println!("  {}  {}", p.created_at.dimmed(), "created as candidate".dimmed());
    for h in &p.history {
        let note = h.note.as_deref().unwrap_or("");
        println!(
            "  {}  {} → {} {}",
            h.date.dimmed(),
            h.from,
            h.to,
            note.dimmed()
        );
    }

    Ok(())
}

pub fn run_add() -> Result<(), Box<dyn std::error::Error>> {
    if !is_initialized() {
        println!("{}", "Canopus not initialized. Run `canopus init` first.".red());
        return Ok(());
    }

    println!("{}", "— New Principle —".cyan().bold());

    let title = read_required("Title: ");
    let description = read_multiline("Description (press Enter twice to finish):");
    if description.trim().is_empty() {
        println!("{}", "Description cannot be empty. Aborted.".yellow());
        return Ok(());
    }
    let domain = read_optional("Domain (optional, press Enter to skip): ");

    println!("{}", "— First evidence entry —".cyan().bold());
    let today = get_today_str();
    let ev_date = {
        let d = read_line(&format!("Date [{}]: ", today));
        if d.is_empty() { today.clone() } else { d }
    };
    let ev_desc = read_required("Evidence description: ");
    let ev_obs = read_optional("Observation id (optional): ");

    let mut index = load_index();
    let id = next_id(&index);
    let num = id.trim_start_matches("principle_").to_string();

    let principle = Principle {
        id: id.clone(),
        title: title.clone(),
        description,
        status: PrincipleStatus::Candidate,
        domain: domain.clone(),
        evidence: vec![Evidence {
            date: ev_date,
            observation_id: ev_obs,
            description: ev_desc,
        }],
        validations: vec![],
        history: vec![],
        created_at: today.clone(),
        updated_at: today,
    };

    save_principle(&principle)?;

    index.principles.push(PrincipleIndexEntry {
        id: id.clone(),
        title,
        status: PrincipleStatus::Candidate,
        domain,
    });
    index.count = index.principles.len() as u32;
    save_index(&index)?;

    println!("{}", format!("Principle #{} created as candidate.", num).green().bold());

    if principle.evidence.len() >= 3 {
        println!(
            "{}",
            format!(
                "You now have enough evidence to validate.\nRun: canopus principles validate {}",
                num
            )
            .cyan()
        );
    }

    Ok(())
}

pub fn run_validate(id_input: String) -> Result<(), Box<dyn std::error::Error>> {
    if !is_initialized() {
        println!("{}", "Canopus not initialized. Run `canopus init` first.".red());
        return Ok(());
    }

    let id = resolve_id(&id_input)?;
    let mut p = load_principle(&id)
        .ok_or_else(|| format!("Principle '{}' not found.", id_input))?;

    if p.status != PrincipleStatus::Candidate {
        return Err(format!("Cannot move from {} to validated.", p.status).into());
    }

    if p.evidence.len() < 3 {
        let needed = 3 - p.evidence.len();
        println!(
            "Need {} more evidence {} before validating.\nUse: canopus principles evidence {}",
            needed,
            if needed == 1 { "entry" } else { "entries" },
            id_input
        );
        return Ok(());
    }

    let num = p.id.trim_start_matches("principle_");
    println!("Principle #{}: \"{}\"", num, p.title);
    if !confirm("Transition to validated?") {
        println!("Aborted.");
        return Ok(());
    }

    let today = get_today_str();
    p.history.push(StatusTransition {
        from: PrincipleStatus::Candidate,
        to: PrincipleStatus::Validated,
        date: today.clone(),
        note: None,
    });
    p.status = PrincipleStatus::Validated;
    p.updated_at = today;
    save_principle(&p)?;
    update_index_status(&id, PrincipleStatus::Validated)?;

    println!("{}", format!("Principle #{} is now validated.", num).green().bold());
    Ok(())
}

pub fn run_confirm(id_input: String) -> Result<(), Box<dyn std::error::Error>> {
    if !is_initialized() {
        println!("{}", "Canopus not initialized. Run `canopus init` first.".red());
        return Ok(());
    }

    let id = resolve_id(&id_input)?;
    let mut p = load_principle(&id)
        .ok_or_else(|| format!("Principle '{}' not found.", id_input))?;

    if p.status != PrincipleStatus::Validated {
        return Err(format!("Cannot move from {} to confirmed.", p.status).into());
    }

    if p.validations.is_empty() {
        println!(
            "Need at least 1 validation record before confirming.\nUse: canopus principles validation {}",
            id_input
        );
        return Ok(());
    }

    let num = p.id.trim_start_matches("principle_");
    println!("Principle #{}: \"{}\"", num, p.title);
    if !confirm("Transition to confirmed?") {
        println!("Aborted.");
        return Ok(());
    }

    let today = get_today_str();
    p.history.push(StatusTransition {
        from: PrincipleStatus::Validated,
        to: PrincipleStatus::Confirmed,
        date: today.clone(),
        note: None,
    });
    p.status = PrincipleStatus::Confirmed;
    p.updated_at = today;
    save_principle(&p)?;
    update_index_status(&id, PrincipleStatus::Confirmed)?;

    println!("{}", format!("Principle #{} is now confirmed.", num).green().bold());
    Ok(())
}

pub fn run_deprecate(id_input: String) -> Result<(), Box<dyn std::error::Error>> {
    if !is_initialized() {
        println!("{}", "Canopus not initialized. Run `canopus init` first.".red());
        return Ok(());
    }

    let id = resolve_id(&id_input)?;
    let mut p = load_principle(&id)
        .ok_or_else(|| format!("Principle '{}' not found.", id_input))?;

    if p.status == PrincipleStatus::Deprecated {
        println!("{}", "Principle is already deprecated.".yellow());
        return Ok(());
    }

    let num = p.id.trim_start_matches("principle_");
    println!(
        "Principle #{}: \"{}\" (currently: {})",
        num, p.title, p.status
    );
    if !confirm("Deprecate this principle?") {
        println!("Aborted.");
        return Ok(());
    }

    let today = get_today_str();
    let from = p.status.clone();
    p.history.push(StatusTransition {
        from,
        to: PrincipleStatus::Deprecated,
        date: today.clone(),
        note: None,
    });
    p.status = PrincipleStatus::Deprecated;
    p.updated_at = today;
    save_principle(&p)?;
    update_index_status(&id, PrincipleStatus::Deprecated)?;

    println!("{}", format!("Principle #{} deprecated.", num).dimmed());
    Ok(())
}

pub fn run_evidence(id_input: String) -> Result<(), Box<dyn std::error::Error>> {
    if !is_initialized() {
        println!("{}", "Canopus not initialized. Run `canopus init` first.".red());
        return Ok(());
    }

    let id = resolve_id(&id_input)?;
    let mut p = load_principle(&id)
        .ok_or_else(|| format!("Principle '{}' not found.", id_input))?;

    println!("{}", format!("Adding evidence to: \"{}\"", p.title).cyan());

    let today = get_today_str();
    let date = {
        let d = read_line(&format!("Date [{}]: ", today));
        if d.is_empty() { today.clone() } else { d }
    };
    let description = read_required("Description: ");
    let observation_id = read_optional("Observation id (optional): ");

    p.evidence.push(Evidence { date, observation_id, description });
    p.updated_at = today;
    save_principle(&p)?;

    let num = p.id.trim_start_matches("principle_");
    println!(
        "{}",
        format!("Evidence added. Total: {} entries.", p.evidence.len()).green()
    );

    if p.evidence.len() >= 3 && p.status == PrincipleStatus::Candidate {
        println!(
            "{}",
            format!(
                "You now have enough evidence to validate.\nRun: canopus principles validate {}",
                num
            )
            .cyan()
        );
    }

    Ok(())
}

pub fn run_validation(id_input: String) -> Result<(), Box<dyn std::error::Error>> {
    if !is_initialized() {
        println!("{}", "Canopus not initialized. Run `canopus init` first.".red());
        return Ok(());
    }

    let id = resolve_id(&id_input)?;
    let mut p = load_principle(&id)
        .ok_or_else(|| format!("Principle '{}' not found.", id_input))?;

    println!(
        "{}",
        format!("Adding validation record to: \"{}\"", p.title).cyan()
    );

    let today = get_today_str();
    let date = {
        let d = read_line(&format!("Date [{}]: ", today));
        if d.is_empty() { today.clone() } else { d }
    };
    let decision = read_required("Decision made using this principle: ");
    let outcome = read_required("Outcome: ");

    p.validations.push(Validation { date, decision, outcome });
    p.updated_at = today;
    save_principle(&p)?;

    let num = p.id.trim_start_matches("principle_");
    println!(
        "{}",
        format!(
            "Validation record added. Total: {} records.",
            p.validations.len()
        )
        .green()
    );

    if p.status == PrincipleStatus::Validated {
        println!(
            "{}",
            format!(
                "You can now confirm this principle.\nRun: canopus principles confirm {}",
                num
            )
            .cyan()
        );
    }

    Ok(())
}

// ── Programmatic creation (called from reflect.rs) ────────────────────────────

/// Creates a new candidate principle without interactive prompts.
/// Returns the zero-padded number string (e.g. "001").
pub fn create_candidate(
    title: String,
    description: String,
    domain: Option<String>,
    evidence_description: String,
    evidence_date: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut index = load_index();
    let id = next_id(&index);
    let num = id.trim_start_matches("principle_").to_string();
    let today = get_today_str();

    let principle = Principle {
        id: id.clone(),
        title: title.clone(),
        description,
        status: PrincipleStatus::Candidate,
        domain: domain.clone(),
        evidence: vec![Evidence {
            date: evidence_date,
            observation_id: None,
            description: evidence_description,
        }],
        validations: vec![],
        history: vec![],
        created_at: today.clone(),
        updated_at: today,
    };

    save_principle(&principle)?;

    index.principles.push(PrincipleIndexEntry {
        id: id.clone(),
        title,
        status: PrincipleStatus::Candidate,
        domain,
    });
    index.count = index.principles.len() as u32;
    save_index(&index)?;

    Ok(num)
}

// ── Public action enum and dispatcher ────────────────────────────────────────

#[derive(Subcommand)]
pub enum PrinciplesAction {
    /// List all principles grouped by status
    List,
    /// Show full detail of one principle
    Show { id: String },
    /// Add a new principle interactively
    Add,
    /// Transition a candidate principle to validated (requires 3 evidence entries)
    Validate { id: String },
    /// Transition a validated principle to confirmed (requires 1 validation record)
    Confirm { id: String },
    /// Deprecate a principle
    Deprecate { id: String },
    /// Add an evidence entry to a principle
    Evidence { id: String },
    /// Add a real-world validation record to a principle
    Validation { id: String },
}

pub fn run(action: PrinciplesAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        PrinciplesAction::List              => run_list(),
        PrinciplesAction::Show { id }       => run_show(id),
        PrinciplesAction::Add               => run_add(),
        PrinciplesAction::Validate { id }   => run_validate(id),
        PrinciplesAction::Confirm { id }    => run_confirm(id),
        PrinciplesAction::Deprecate { id }  => run_deprecate(id),
        PrinciplesAction::Evidence { id }   => run_evidence(id),
        PrinciplesAction::Validation { id } => run_validation(id),
    }
}
