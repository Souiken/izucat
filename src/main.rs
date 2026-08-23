use clap::{Arg, Command};
use ignore::WalkBuilder;
use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::Command as ProcessCommand;
use std::time::Instant;


/// binary?
fn is_binary(path: &Path) -> bool {
    let Ok(mut f) = File::open(path) else { return true };
    let mut buffer = [0; 1024];
    let Ok(n) = f.read(&mut buffer) else { return true };
    let chunk = &buffer[..n];

    chunk.contains(&0) || chunk.iter().any(|&b| b < 0x20 && ![7, 8, 9, 10, 12, 13, 27].contains(&b))
}

/// escape
fn escape_typst(text: &str) -> String {
    text.to_string()
}

/// Process a single file and write to output
fn process_file(out: &mut File, path: &Path, display_name: &str, line_num: bool) -> io::Result<()> {
    writeln!(out, "#codeblock(\u{0060}\u{0060}\u{0060}\u{0060}")?;

    let mut data = Vec::new();
    File::open(path)?.read_to_end(&mut data)?;

    let (content, show_line_num) = if is_binary(path) {
        (to_hex_view(&data), false)
    } else {
        match String::from_utf8(data.clone()) {
            Ok(content) => (content, line_num),
            Err(_) => (to_hex_view(&data), false),
        }
    };

    let suffix = if !show_line_num && line_num { " (binary)" } else if !show_line_num { " (non-UTF8)" } else { "" };
    writeln!(out, "{}{}\n----------------", display_name, suffix)?;
    writeln!(out, "{}", escape_typst(&content))?;
    writeln!(out, "\u{0060}\u{0060}\u{0060}\u{0060}, {})\n", show_line_num)?;
    Ok(())
}

/// bin to hex
fn to_hex_view(data: &[u8]) -> String {
    let mut lines = Vec::new();
    lines.push("Hex View  00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F".to_string());

    for (i, chunk) in data.chunks(16).enumerate() {
        let offset = i * 16;
        let hex_bytes: Vec<String> = chunk.iter().map(|b| format!("{:02X}", b)).collect();
        let ascii_bytes: String = chunk
            .iter()
            .map(|&b| {
                if (32..=126).contains(&b) {
                    b as char
                } else {
                    '.'
                }
            })
            .collect();
        lines.push(format!(
            "{:08X}  {:<47}  {}",
            offset,
            hex_bytes.join(" "),
            ascii_bytes
        ));
    }

    lines.join("\n")
}

/// generate typst
fn generate_typst(input_dir: Option<&str>, output_file: &str, line_num: bool, line_num_black: bool , cmd_args: Option<Vec<String>>, use_stdin: bool) -> io::Result<()> {
    let start_time = Instant::now();

    if cmd_args.is_none() && input_dir == Option::from("none") && !use_stdin {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "either an input directory or --cmd must be specified"));
    }

    let mut out = File::create(output_file)?;

    let line_num_colour = if line_num_black { "black" } else { "gray" };

    writeln!(out, "#set page(width: 210mm, height: 297mm, margin: 2cm)")?;
    writeln!(out, "#show raw: set text(font: \"Unifont\", size: 8pt)")?;
    writeln!(out, "#show raw: set par(leading: 0.46em)\n")?;
    writeln!(out, "#let codeblock(code, lineNum) = {{\n  if lineNum {{\n    show raw.line: it => {{\n      box(\n        stack(\n          dir: ltr,\n          box(\n            width: 0em,\n            align(right, \n              text(fill: {line_num_colour})[\n                #if it.number >= 3 {{ (it.number - 2) }} else {{ \"\" }}\n              ]\n            )\n          ),\n          h(1em),\n          it.body,\n        ),\n      )\n    }}\n    code\n  }}\n  else{{code}}\n}}")?;

    if use_stdin {
        println!("\r      izucat v{}", env!("CARGO_PKG_VERSION"));
        println!(
            "\r  \x1b[1;32mGenerating\x1b[0m {} form stdin",
            output_file
        );

        use std::io::BufRead;
        let stdin = io::stdin();
        let mut content = String::new();
        for line in stdin.lock().lines() {
            content.push_str(&line?);
            content.push('\n');
        }

        writeln!(out, "#codeblock(\u{0060}\u{0060}\u{0060}\u{0060}\nstdin (piped)\n----------------")?;
        writeln!(out, "{}", escape_typst(&content))?;
        writeln!(out, "\u{0060}\u{0060}\u{0060}\u{0060}, {})\n", if line_num { "true" } else { "false" })?;

        let duration = start_time.elapsed();
        println!(
            "\r    \x1b[1;32mFinished\x1b[0m stdin in {:.2?}{}",
            duration,
            " ".repeat(40),
        );

    } else if let Some(cmd) = cmd_args {
        println!("\r      izucat v{}", env!("CARGO_PKG_VERSION"));
        println!(
            "\r  \x1b[1;32mGenerating\x1b[0m {} `{}`",
            output_file, cmd.join("")
        );
        let output = ProcessCommand::new(&cmd[0])
            .args(&cmd[1..])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let escaped = escape_typst(&format!("{}\n{}", stdout, stderr));

        writeln!(out, "#codeblock(\u{0060}\u{0060}\u{0060}\u{0060}\n$ {}\n----------------\n{}\n\u{0060}\u{0060}\u{0060}\u{0060}, {})", cmd.join(" "), escaped, line_num)?;

        let duration = start_time.elapsed();
        println!(
            "\r    \x1b[1;32mFinished\x1b[0m run `{}` in {:.2?}{}",
            cmd.join(" "),
            duration,
            " ".repeat(40),
        );

    } else if let Some(input_dir) = input_dir {
        println!("\r      izucat v{}", env!("CARGO_PKG_VERSION"));
        println!(
            "\r  \x1b[1;32mGenerating\x1b[0m {} ({})",
            output_file, input_dir
        );
        let path = Path::new(input_dir);
        if path.is_file() {
            let display_name = path.file_name().unwrap().to_string_lossy();
            process_file(&mut out, path, &display_name, line_num)?;
        } else {
            let mut entries: Vec<_> = WalkBuilder::new(input_dir)
                .standard_filters(true)
                .build()
                .filter_map(Result::ok)
                .filter(|e| e.path().is_file())
                .collect();

            let total = entries.len();
            entries.sort_by_key(|e| e.file_name().to_os_string());

            let bar = ProgressBar::new(entries.len() as u64);
            bar.set_style(
                ProgressStyle::default_bar()
                    .template("\r    \x1b[1;36mBuilding\x1b[0m [{bar:25}] {pos}/{len}: {msg}")
                    .unwrap()
                    .progress_chars("=> "),
            );

            for (i, entry) in entries.iter().enumerate() {
                bar.set_message(entry.path().display().to_string());
                let path = entry.path();
                let rel_path = path.strip_prefix(input_dir).unwrap_or(path);
                let display_name = rel_path.display().to_string().replace("\\\\", "/");
                process_file(&mut out, path, &display_name, line_num)?;

                if i < entries.len() - 1 {
                    writeln!(out, "#pagebreak()")?;
                }
                bar.inc(1);
            }

            bar.finish_with_message("Done!");
            let duration = start_time.elapsed();
            println!(
                "\r    \x1b[1;32mFinished\x1b[0m {} file(s) in {:.2?}{}",
                total,
                duration,
                " ".repeat(40),
            );
        }
    }

    println!("   \x1b[1;32mGenerated\x1b[0m {}", output_file);
    println!(
        "             run `typst c {} output.pdf` for pdf",
        output_file
    );
    Ok(())
}

/// program entrance
fn main() -> Result<(), ()> {
    let matches = Command::new("izucat")
        .about("A program that can recursively concatenate (cat) text and binary files in a path and/or command/stdin output to typst. ")
        .arg(
            Arg::new("output")
                .short('o')
                // .long("output")
                .value_name("FILE")
                .required(false)
                .help("Sets the output file name (default: output.typ)"),
        )
        .arg(
            Arg::new("input")
                .help("Sets the input path (Optional)")
                .required(false)
                .index(1),
        )
        .arg(
            Arg::new("noLineNumbers")
                .short('n')
                .long("no-line-numbers")
                .help("Sets not show line numbers for text.")
                .required(false)
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("blackLineNumbers")
                .short('b')
                .long("black-line-numbers")
                .help("Sets line numbers to black for Dot Matrix Printers.")
                .required(false)
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("cmd")
                .long("cmd")
                .num_args(1..)
                .help("Command to run and capture output"),
        )
        .after_help(
            "\x1b[4m\x1b[1mExamples:\x1b[0m
  \x1b[1mizucat\x1b[0m ./src
  \x1b[1mizucat\x1b[0m \x1b[1m--cmd\x1b[0m \"make\"
  \x1b[1mizucat\x1b[0m ./src \x1b[1m--cmd\x1b[0m \"make\"
  \x1b[1mecho\x1b[0m hello | \x1b[1mizucat\x1b[0m",
        )
        .get_matches();

    let input_path = matches
        .get_one::<String>("input")
        .cloned()
        .unwrap_or_else(|| "none".to_string());
    let output_file = matches
        .get_one::<String>("output")
        .cloned()
        .unwrap_or_else(|| "output.typ".to_string());
    let line_num = matches.get_flag("noLineNumbers");
    let line_num_black = matches.get_flag("blackLineNumbers");
    let cmd_args = matches
        .get_many::<String>("cmd")
        .map(|vals| vals.cloned().collect::<Vec<_>>());
    let use_stdin = atty::isnt(atty::Stream::Stdin) && input_path == "none" && cmd_args.is_none();

    if let Err(e) = generate_typst(Some(&input_path), &output_file, !line_num, line_num_black, cmd_args, use_stdin) {
        eprintln!("\x1b[1;31merror\x1b[0m\x1b[1m:\x1b[0m {}", e);
        std::process::exit(1);
    } else {
        Ok(())
    }
}
