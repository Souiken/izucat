use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use walkdir::WalkDir;

/// binary?
fn is_binary(path: &Path) -> bool {
    if let Ok(mut f) = File::open(path) {
        let mut buffer = [0; 1024];
        if let Ok(n) = f.read(&mut buffer) {
            let chunk = &buffer[..n];
            if chunk.contains(&0) {
                return true;
            }

            let text_characters: Vec<u8> = (0x20..=0xFF)
            .chain([7, 8, 9, 10, 12, 13, 27])
            .collect();
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
    text.replace('`', "\\`")
    // .replace('\\', "\\\\")
    // .replace('#', "\\#")
    // .replace('`', "\\`")
    // .replace('[', "\\[")
    // .replace(']', "\\]")
    // .replace('*', "\\*")
    // .replace('$', "\\$")
    // .replace('<', "\\<")
    // .replace('_', "\\_")
    // .replace('@', "\\@")

    // text.to_string()
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
        .map(|&b| if (32..=126).contains(&b) { b as char } else { '.' })
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

/// generate typst from path
fn generate_typst_from_dir(input_dir: &str, output_file: &str) -> io::Result<()> {
    let mut entries: Vec<_> = WalkDir::new(input_dir)
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(|e| e.path().is_file())
    .collect();

    entries.sort_by_key(|e| e.file_name().to_os_string());

    let mut out = File::create(output_file)?;

    writeln!(out, "#set page(width: 210mm, height: 297mm, margin: 2cm)")?;
    writeln!(out, "#show raw: set text(font: \"Unifont\", size: 8pt)")?;
    writeln!(out, "#show raw: set par(leading: 0.46em)\n")?;

    for (i, entry) in entries.iter().enumerate() {
        let path = entry.path();
        let rel_path = path.strip_prefix(input_dir).unwrap_or(path);
        let mut display_name = rel_path.display().to_string().replace("\\", "/");

        if is_binary(path) {
            display_name += " (binary)";
            let mut data = Vec::new();
            File::open(path)?.read_to_end(&mut data)?;
            let hex_view = to_hex_view(&data);
            let escaped = escape_typst(&hex_view);

            writeln!(out, "```text")?;
            writeln!(out, "{}\n----------------", display_name)?;
            writeln!(out, "{}", escaped)?;
            writeln!(out, "```\n")?;
        } else {
            let mut content = String::new();
            File::open(path)?.read_to_string(&mut content)?;
            let escaped = escape_typst(&content);

            writeln!(out, "```text")?;
            writeln!(out, "{}\n----------------", display_name)?;
            writeln!(out, "{}", escaped)?;
            writeln!(out, "```\n")?;
        }

        if i < entries.len() - 1 {
            writeln!(out, "#pagebreak()\n")?;
        }
    }

    println!("Generate OK: {}", output_file);
    println!("Use typst c {} output.pdf", output_file);
    Ok(())
}

/// program entrance
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 || args[2] != "-o" {
        eprintln!("Usage: izucat <path> -o <output>");
        std::process::exit(1);
    }

    let input_dir = &args[1];
    let output_file = &args[3];

    if let Err(e) = generate_typst_from_dir(input_dir, output_file) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
