# izucat
A program that can recursively concatenate (cat) text and binary files in a path to typst.

## Build
```bash
git clone https://github.com/souiken/izucat.git
cd izucat
cargo build --release
```

## Usage
```
Usage: izucat -o <FILE> <input>

Arguments:
  <input>  Sets the input path

Options:
  -o <FILE>      Sets the output file name
  -h, --help     Print help

```

Example:
```
izucat -o output.typ ./project 
typst c output.typ output.pdf
```

## Output
Text:
```text
path/to/file
----------------
Everyone has the right to an effective remedy by the competent national tribunals
for acts violating the fundamental rights granted him by the constitution or by law. 
```

HEX:
```text
path/to/file  (binary)
----------------
Hex View 00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F
00000000 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 ................
00000010 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 ................
00000020 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 ................
```

