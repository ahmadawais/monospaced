use std::env;
use std::io::{self, IsTerminal, Read, Write};
use std::process::{self, Command, Stdio};
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
        report_clipboard_result(copy_to_clipboard(&converted));
        println!("{converted}");
        return Ok(());
    }

    if !io::stdin().is_terminal() {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input).map_err(io_error)?;

        let converted = with_spinner("Converting", || to_monospace(&input));
        report_clipboard_result(copy_to_clipboard(&converted));
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
        let clipboard_result = copy_to_clipboard(&converted);
        println!();
        println!("{}", style("1", "Monospaced output", colors));
        println!("{converted}");
        report_clipboard_result(clipboard_result);
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

fn report_clipboard_result(result: Result<(), String>) {
    if !io::stderr().is_terminal() {
        return;
    }

    match result {
        Ok(()) => eprintln!("{}", style("2", "Copied to clipboard.", true)),
        Err(message) => eprintln!("{}", style("2", &format!("Clipboard unavailable: {message}"), true)),
    }
}

fn copy_to_clipboard(text: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        return pipe_to_command("pbcopy", &[], text);
    }

    #[cfg(target_os = "windows")]
    {
        return pipe_to_command("cmd", &["/C", "clip"], text);
    }

    #[cfg(target_os = "linux")]
    {
        return copy_to_linux_clipboard(text);
    }

    #[allow(unreachable_code)]
    Err("clipboard copy is unsupported on this platform".to_owned())
}

#[cfg(target_os = "linux")]
fn copy_to_linux_clipboard(text: &str) -> Result<(), String> {
    let candidates: [(&str, &[&str]); 3] = [
        ("wl-copy", &[]),
        ("xclip", &["-selection", "clipboard"]),
        ("xsel", &["--clipboard", "--input"]),
    ];

    let mut last_error = None;
    for (command, args) in candidates {
        match pipe_to_command(command, args, text) {
            Ok(()) => return Ok(()),
            Err(error) => last_error = Some(error),
        }
    }

    Err(last_error.unwrap_or_else(|| "install wl-copy, xclip, or xsel".to_owned()))
}

fn pipe_to_command(command: &str, args: &[&str], text: &str) -> Result<(), String> {
    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| error.to_string())?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .map_err(|error| error.to_string())?;
    }

    let status = child.wait().map_err(|error| error.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "{command} exited with {}",
            status
                .code()
                .map_or_else(|| "no exit code".to_owned(), |code| code.to_string())
        ))
    }
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
