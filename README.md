# izucat
A program that can recursively concatenate (cat) text and binary files in a path and/or command/stdin output to typst.

## Build
```bash
git clone https://github.com/souiken/izucat.git
cd izucat
cargo build --release
```

## Usage
```
A program that can recursively concatenate (cat) text and binary files in a path and/or command/stdin output to typst. 

Usage: izucat [OPTIONS] -o <FILE> [input]

Arguments:
  [input]  Sets the input path (Optional)

Options:
  -o <FILE>              Sets the output file name
      --no-line-numbers  Sets not show line numbers for text.
      --cmd <cmd>...     Command to run and capture output
  -h, --help             Print help

Examples:
  izucat -o output.typ ./src
  izucat -o output.typ --cmd "make"
  izucat -o output.typ ./src --cmd "make"
  echo hello | izucat -o output.typ
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

Command:
```text
$ echo hello
----------------
hello
```

Stdio:

```text
stdin (piped)
----------------
hello
```
