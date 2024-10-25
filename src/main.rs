use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use qqwing::{difficulty::Difficulty, PrintStyle, QQWing};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Input or Output puzzle file
    #[arg(short, long, value_name = "FILE")]
    file: Option<PathBuf>,

    /// Show more verbose information
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Set print style
    #[arg(
        short,
        long,
        value_name = "ONELINE,COMPACT,READABLE,CSV",
        default_value = "READABLE"
    )]
    ps: Option<PrintStyle>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a puzzle
    Generate {
        /// number of puzzles to generate
        #[arg(short, long, default_value = "1")]
        nums: u32,

        /// puzzle difficulty level to generate
        #[arg(
            short,
            long,
            value_name = "UNKNOWN,SIMPLE,EASY,MEDIUM,EXPERT",
            default_value = "UNKNOWN"
        )]
        difficulty: Difficulty,
    },
    /// Solve a puzzle
    Solve {
        /// Print the puzzle stats
        #[arg(short, long)]
        stats: bool,
        /// Print the puzzle
        #[arg(short, long)]
        puzzle: String,
    },
}

fn main() {
    let cli = Cli::parse();
    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    let max_level = match cli.verbose {
        0 => Level::WARN,
        1 => Level::INFO,
        _ => Level::DEBUG,
    };
    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(max_level)
        .with_file(true)
        .with_line_number(true)
        // completes the builder.
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    let mut ss = QQWing::new();

    ss.set_print_style(cli.ps.unwrap());

    if let Some(file_path) = cli.file.as_deref() {
        println!("Value for file: {}", file_path.display());
    }

    ss.set_log_history(true);
    ss.set_record_history(true);

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Generate { nums, difficulty } => {
            info!("Set puzzle difficulty level {:?} to generate", difficulty);
            let num = *nums;
            info!("Start generate puzzle");
            let mut n = 0;
            while n < num {
                ss.generate_puzzle();
                ss.set_record_history(true);
                ss.solve();
                if *difficulty == Difficulty::UNKNOWN || ss.get_difficulty() == *difficulty {
                    info!(
                        "get a puzzle with difficulty {:?}, print it:",
                        ss.get_difficulty()
                    );
                    ss.print_puzzle();
                    n += 1;
                } else {
                    info!(
                        "get a puzzle with difficulty {:?} != {:?}, continue generate...",
                        ss.get_difficulty(),
                        *difficulty
                    );
                }
            }
        }
        Commands::Solve { stats, puzzle } => {
            if puzzle.len() == qqwing::BOARD_SIZE {
                info!("Set the puzzle");
                let init_puzzle = read_puzzle(puzzle);
                ss.set_puzzle(init_puzzle);
            }
            info!("Start solve puzzle");
            if ss.solve() {
                ss.print_solve_instructions();
            }
            if *stats {
                println!("{}", ss.get_stats());
            }
        }
    }
}

/**
 * Read a sudoku puzzle from a String input. Any digit is
 * used to fill the sudoku, any other character is ignored.
 */
fn read_puzzle(puzzle_str: &str) -> Vec<u8> {
    let mut puzzle = Vec::new();
    for c in puzzle_str.chars() {
        let n = c.to_digit(10);
        match n {
            Some(n) => puzzle.push(n as u8),
            None => puzzle.push(0),
        }
    }
    puzzle
}
