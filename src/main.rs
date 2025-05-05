use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::Instant;

/// binary?
fn is_binary(path: &Path) -> bool {
    if let Ok(mut f) = File::open(path) {
        let mut buffer = [0; 1024];
        if let Ok(n) = f.read(&mut buffer) {
            let chunk = &buffer[..n];
            if chunk.contains(&0) {
                return true;
            }

            let text_characters: Vec<u8> = (0x20..=0xFF).chain([7, 8, 9, 10, 12, 13, 27]).collect();
            let nontext: Vec<u8> = chunk
                .iter()
                .filter(|b| !text_characters.contains(b))
                .copied()
                .collect();
            return !nontext.is_empty();
        }
    }
    true
}

/// escape
fn escape_typst(text: &str) -> String {
    text.replace('`', "`\u{200B}")
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
fn generate_typst(input_dir: &str, output_file: &str) -> io::Result<()> {
    let start_time = Instant::now();
    let mut entries: Vec<_> = ignore::WalkBuilder::new(input_dir)
        .build()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();

    let total = entries.len();
    entries.sort_by_key(|e| e.file_name().to_os_string());

    println!("\r      izucat v{}", env!("CARGO_PKG_VERSION"));
    println!("\r  \x1b[1;92mGenerating\x1b[0m {} ({})", output_file, input_dir);

    let mut out = File::create(output_file)?;

    writeln!(out, "#set page(width: 210mm, height: 297mm, margin: 2cm)")?;
    writeln!(out, "#show raw: set text(font: \"Unifont\", size: 8pt)")?;
    writeln!(out, "#show raw: set par(leading: 0.46em)\n")?;

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
        let mut display_name = rel_path.display().to_string().replace("\\", "/");

        writeln!(out, "```text")?;

        if is_binary(path) {
            display_name += " (binary)";
            let mut data = Vec::new();
            File::open(path)?.read_to_end(&mut data)?;
            let hex_view = to_hex_view(&data);
            let escaped = escape_typst(&hex_view);

            writeln!(out, "{}\n----------------", display_name)?;
            writeln!(out, "{}", escaped)?;
        } else {
            let mut content = String::new();
            File::open(path)?.read_to_string(&mut content)?;
            let escaped = escape_typst(&content);

            writeln!(out, "{}\n----------------", display_name)?;
            writeln!(out, "{}", escaped)?;
        }

        writeln!(out, "```\n")?;

        if i < entries.len() - 1 {
            writeln!(out, "#pagebreak()")?;
        }
        bar.inc(1);
    }
    bar.finish_with_message("Done!");
    let duration = start_time.elapsed();
    println!("\r    \x1b[1;92mFinished\x1b[0m {} files in {:.2?}{}",total,duration," ".repeat(40),);

    println!("   \x1b[1;92mGenerated\x1b[0m {}", output_file);
    println!("             run `typst c {} output.pdf` for pdf", output_file);
    Ok(())
}

/// program entrance
fn main() -> std::io::Result<()> {
    let _args = env::args().skip(1);
    let mut input_path = None;
    let mut output_file = None;

    // help 
    let args_vec: Vec<String> = env::args().skip(1).collect();
    if args_vec.is_empty() || args_vec.iter().any(|a| a == "-h" || a == "--help") {
        println!("Usage: izucat [OPTIONS] <INPUT_DIR>\nOptions:\n    -o <FILE>        Output Typst file name (default: output.typ)\n    -h, --help       Show this help message
        ");
        return Ok(());
    }

    let mut args = args_vec.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" => {
                output_file = args.next();
            }
            _ => {
                input_path = Some(arg);
            }
        }
    }

    let input_path = input_path.unwrap_or_else(|| ".".to_string());
    let output_file = output_file.unwrap_or_else(|| "output.typ".to_string());

    generate_typst(&input_path, &output_file)?;

    Ok(())
}
