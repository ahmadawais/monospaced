use std::env;
use std::io::{self, IsTerminal, Read, Write};
use std::process;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use monospaced::to_monospace;

const WIDE_BANNER: [&str; 4] = [
    "███╗   ███╗ ██████╗ ███╗  ██╗ ██████╗  ███████╗██████╗  █████╗  ██████╗███████╗██████╗ ",
    "████╗ ████║██╔═══██╗████╗ ██║██╔═══██╗ ██╔════╝██╔══██╗██╔══██╗██╔════╝██╔════╝██╔══██╗",
    "██╔████╔██║██║   ██║██╔██╗██║██║   ██║ ███████╗██████╔╝███████║██║     █████╗  ██║  ██║",
    "██║╚██╔╝██║╚██████╔╝██║ ╚████║╚██████╔╝ ╚██████║██║     ██║  ██║╚██████╗███████╗██████╔╝",
];

const COMPACT_BANNER: [&str; 4] = [
    "███╗   ███╗ ██████╗ ███╗  ██╗ ██████╗ ",
    "████╗ ████║██╔═══██╗████╗ ██║██╔═══██╗",
    "██╔████╔██║██║   ██║██╔██╗██║██║   ██║",
    "██║╚██╔╝██║╚██████╔╝██║ ╚████║╚██████╔╝",
];

enum AppError {
    Message(String),
    Usage(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Message(message) | Self::Usage(message) => f.write_str(message),
        }
    }
}

impl AppError {
    fn exit_code(&self) -> i32 {
        match self {
            Self::Usage(_) => 2,
            Self::Message(_) => 1,
        }
    }
}

type AppResult<T> = Result<T, AppError>;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(error.exit_code());
    }
}

fn run() -> AppResult<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();

    match args.first().map(String::as_str) {
        Some("-h") | Some("--help") => {
            print_help();
            Ok(())
        }
        Some("-v") | Some("--version") => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some("convert") => run_subcommand(&args[1..]),
        Some(_) => {
            let text_parts = parse_text_parts(&args)?;
            run_convert_command(&text_parts)
        }
        None => run_convert_command(&[]),
    }
}

fn run_subcommand(args: &[String]) -> AppResult<()> {
    match args.first().map(String::as_str) {
        Some("-h") | Some("--help") => {
            print_convert_help();
            Ok(())
        }
        Some(_) => {
            let text_parts = parse_text_parts(args)?;
            run_convert_command(&text_parts)
        }
        None => run_convert_command(&[]),
    }
}

fn parse_text_parts(args: &[String]) -> AppResult<Vec<String>> {
    let mut text_parts = Vec::with_capacity(args.len());
    let mut allow_leading_dash = false;

    for arg in args {
        if allow_leading_dash {
            text_parts.push(arg.clone());
            continue;
        }

        if arg == "--" {
            allow_leading_dash = true;
            continue;
        }

        if arg.starts_with('-') {
            return Err(AppError::Usage(format!(
                "error: unknown option '{arg}'\n(use -- to pass text that starts with '-')"
            )));
        }

        text_parts.push(arg.clone());
    }

    Ok(text_parts)
}

fn run_convert_command(text_parts: &[String]) -> AppResult<()> {
    if !text_parts.is_empty() {
        let input = text_parts.join(" ");
        let converted = with_spinner("Converting", || to_monospace(&input));
        println!("{converted}");
        return Ok(());
    }

    if !io::stdin().is_terminal() {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input).map_err(io_error)?;

        let converted = with_spinner("Converting", || to_monospace(&input));
        print!("{converted}");
        io::stdout().flush().map_err(io_error)?;
        return Ok(());
    }

    run_interactive_prompt()
}

fn run_interactive_prompt() -> AppResult<()> {
    let colors = io::stdout().is_terminal();
    println!("{}", render_banner(colors));
    println!(
        "{}",
        style("2", "Turn plain text into Unicode monospace text.", colors)
    );

    loop {
        print!("Text to convert: ");
        io::stdout().flush().map_err(io_error)?;

        let mut input = String::new();
        let read = io::stdin().read_line(&mut input).map_err(io_error)?;
        if read == 0 {
            println!();
            return Ok(());
        }

        let input = input.trim_end_matches(['\n', '\r']);
        if input.trim().is_empty() {
            eprintln!("Enter some text to convert.");
            continue;
        }

        let converted = with_spinner("Converting", || to_monospace(input));
        println!();
        println!("{}", style("1", "Monospaced output", colors));
        println!("{converted}");
        println!("{}", style("2", "Done.", colors));
        return Ok(());
    }
}

fn with_spinner<T, F>(label: &str, task: F) -> T
where
    F: FnOnce() -> T,
{
    if !io::stderr().is_terminal() {
        return task();
    }

    let running = Arc::new(AtomicBool::new(true));
    let spinner_flag = Arc::clone(&running);
    let spinner_label = label.to_owned();

    let handle = thread::spawn(move || {
        let frames = ['-', '\\', '|', '/'];
        let mut index = 0usize;

        while spinner_flag.load(Ordering::Relaxed) {
            eprint!("\r[{}] {}", frames[index % frames.len()], spinner_label);
            let _ = io::stderr().flush();
            thread::sleep(Duration::from_millis(80));
            index += 1;
        }
    });

    let result = task();
    running.store(false, Ordering::Relaxed);
    let _ = handle.join();
    eprint!("\r[ok] {label}\n");
    let _ = io::stderr().flush();

    result
}

fn render_banner(colors: bool) -> String {
    let banner = if terminal_columns() >= 120 {
        &WIDE_BANNER
    } else {
        &COMPACT_BANNER
    };

    banner
        .iter()
        .enumerate()
        .map(|(index, line)| {
            if index == 0 {
                style("37", line, colors)
            } else {
                style("2", line, colors)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn terminal_columns() -> usize {
    env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(80)
}

fn style(code: &str, text: &str, enabled: bool) -> String {
    if enabled {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_owned()
    }
}

fn io_error(error: io::Error) -> AppError {
    AppError::Message(error.to_string())
}

fn print_help() {
    println!(
        "\
Turn plain text into Unicode monospace text.

Usage:
  monospaced [text...]
  monospaced convert [text...]

Options:
  -h, --help       Print help
  -v, --version    Print version

Examples:
  monospaced \"npx create-next-app 14\"
  echo \"npx create-next-app 14\" | monospaced
  monospaced convert \"hello 123\""
    );
}

fn print_convert_help() {
    println!(
        "\
Convert plain text into Unicode monospace text.

Usage:
  monospaced convert [text...]

Options:
  -h, --help    Print help

Examples:
  monospaced convert \"hello 123\"
  echo \"hello 123\" | monospaced convert"
    );
}
