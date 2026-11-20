# bni-rs

A parser, linter and semantic highlighter for **BNI** (Blizzard INI), the
INI-like data-file format of classic Blizzard games such as
*Warcraft III*. It's built like the other language crates, on the shared
[`parser-rs`](../parser-rs), and replaces the earlier
[`tree-sitter-bni`](../tree-sitter-bni) grammar.

BNI files are found throughout the game's data and in World Editor mods:
`TriggerData.txt`, `TriggerStrings.txt`, `WorldEditStrings.txt`,
`WorldEditData.txt`, `MiscData.txt`, `MiscUI.txt`, `CampaignStrings.txt`,
the `*Func.txt` / `*Strings.txt` object-data files, and many more.

- [The BNI format](#the-bni-format): a syntax reference to build on
- [Parser status](#parser-status): what `bni-rs` handles today
- [How it differs from the other parsers](#how-it-differs-from-the-other-parsers)
- [CLI](#cli) · [Tests](#tests) · [Library use](#library-use)

How the parsers are built, and the rules every parser here follows, is in
the [`parser-rs` README](../parser-rs/README.md#principles).

---

## The BNI format

### Lines

A BNI file is **line-oriented**. Every line is read on its own and is
exactly one of:

| line           | syntax                    | example                                     |
|----------------|---------------------------|---------------------------------------------|
| section header | `[Name]`                  | `[TriggerCategories]`                       |
| item           | `key=value`               | `FadeBuffMinDuration=10`                    |
| comment        | `// ...`                  | `//*************************************`   |
| empty          | only spaces and tabs      |                                             |

- **Indentation.** A section header or comment may be indented. On an
  item line, leading spaces belong to the key (see below).
- **Line endings.** `\n` and `\r\n` both work. A leading UTF-8 byte-order
  mark is ignored.
- **Malformed lines.** A line that is none of the above, typically text
  with no `=`, is an error, and only that line is affected.

### Sections

```ini
[TriggerCategories]
[Blight]
```

- **The name** is everything between `[` and `]`, as written and
  case-sensitive.
- **The closing `]` is optional:** `[Section` is accepted. The name then
  runs to the end of the line, without trailing spaces.
- **After `]`** only spaces and a `// comment` may follow.
- **Items** belong to the nearest section header above them.

### Items

```ini
WESTRING_APPNAME=Warcraft III World Editor
GoldTextColor=255,255,220,0
```

- **Key and value.** The key is everything before the **first** `=`,
  **exactly as written**, spaces included. The value is everything after
  it, so `K=a=b` has the value `a=b`.
- **Optional sides.** Both the key and the value may be missing:

  | syntax    | meaning                 |
  |-----------|-------------------------|
  | `Key=val` | key with a value        |
  | `Key=`    | key with an empty value |
  | `=val`    | no key, just a value    |
  | `=`       | bare equals sign        |

- **Keys aren't trimmed, on purpose.** UMSWE names its "custom script"
  trigger entries with keys made **only of spaces**, a different number
  for each entry. What looks like an indented keyless line is such a key,
  and its other entries spell it out between underscores:

  ```ini
       =0,1,abilcode,scriptcode
  _     _Category=TC_CUSTOM
  ```

- **Repeated keys are normal.** In `TriggerStrings.txt` a key appears up
  to three times: the display name, the format with parameters, then the
  hint.

  ```ini
  MapInitializationEvent=Map Initialization
  MapInitializationEvent="Map initialization"
  ```

### Values

A value is a list of **fields** separated by commas. Each field is one or
more **parts**:

| part            | examples                               | rule                                             |
|-----------------|----------------------------------------|--------------------------------------------------|
| integer         | `42`, `-1`, `0`                        | the whole part is `-?digits` (and fits 64 bits)  |
| real            | `0.5`, `-3.14`, `0.00`, `.5`           | the whole part is `-?digits.digits`, digits on at least one side |
| quoted string   | `"City Building (Diagonal 1, Red)"`    | `"` to the next `"` on the same line             |
| unquoted text   | `Warcraft III`, `Kel'Thuzad (Lich)`    | anything else up to `,`, `"`, `//` or line end   |

```ini
GoldTextColor=255,255,220,0          → int, int, int, int
GoldTextVelocity=0,0.03,100          → int, real, int
abilcode=0,1,1,WESTRING_TRIGTYPE_abilcode,integer
```

- **Whitespace around a part isn't part of it.** In `Key= a b ,c` the
  first part is `a b`. Whitespace *inside* unquoted text is kept.
- **A part is a number only as a whole.** `12abc`, `1e5`, `1.2.3` and a
  lone `-` are text.
- **Empty fields exist.** `a,,b` has three fields, the middle one empty,
  and `a,` has two.
- **Adjacent parts form one field.** `a"b, c"d` is one field with the
  text `ab, cd`. It's rare, but it's how the format behaves.

### Quoted strings

Quote a value when it contains a comma, or `//`, that isn't a separator:

```ini
Name="City Building (Diagonal 1, Red)"
EndCinematic="Cinematic","Arthas' Betrayal","HumanEd"
FuncKey=~Player," ",~Presses/Releases," the ",~Key," key"
```

- **Double quotes only.** `'` is an ordinary character:
  `QuickSave='S'` is the unquoted text `'S'`.
- **No escape sequences.** A backslash is literal, so
  `WESTRING_HOTKEY_BACKSLASH="\"` is a one-character string `\`.
- **No line breaks.** A string can't span lines. An unclosed `"` runs to
  the end of the line.
- **`""`** is an empty string.

### Comments

`//` starts a comment anywhere outside a quoted string: on its own line,
after a section header, or after a value.

```ini
// standalone
PuffDuration=3000 // Duration of a puff, in milliseconds
QuickSave='S'           // Alt-S
```

There are no block comments, and `;` and `#` are ordinary characters,
not comments as in Windows INI.

### Conventions inside values

These aren't syntax, but they're common in the World Editor's data and
are highlighted (see below):

- **`~Name`** is a parameter placeholder in `TriggerStrings.txt` format
  strings, as in `~Player`.
- **`WESTRING_*`** refers to an entry of `WorldEditStrings.txt`.
- **`TRIGSTR_*`** refers to a map's string table (`war3map.wts`).

---

## Parser status

Supported: everything in the reference above. The real UMSWE and UJAPI
`TriggerData.txt` / `TriggerStrings.txt` files (60k lines) parse with no
malformed lines; there's a test for that when
[`tree-sitter-bni`](../tree-sitter-bni) is checked out next to this crate.

The AST (`ast.rs`):

```
Document
├── preamble: [Item]          items before the first header
├── sections: [Section]       name, line, closed (has `]`), items
│   └── Item                  key: Option<Name>, eq, fields
│       └── Field             parts, span (zero-width when empty)
│           └── Value         kind (Int / Real / Text / Quoted{closed}), text, span
└── errors: [ParseError]      one per malformed line
```

**Errors** (in `Document::errors`, with exact columns):

- a line that is neither a section, an item, nor a comment
  (`expected '=' in '...'`);
- text after a section header's `]`.

Every lint diagnostic names its rule. The rule ids, which rules are on
by default (`infinite-loop` isn't) and how a project switches them
(`[lint]` in `wc3.bni`) are in the [`wc3-rs` README](../wc3-rs/README.md#lint).

**Lint warnings** (`lint`). Only near-certain mistakes are reported.
Repeated keys and space-only keys are **not**, because both are normal
(see above).

- items before the first section header;
- a section header missing its `]`;
- an empty section name `[]`;
- a duplicate section header;
- an unterminated quoted string. This one catches real typos: UMSWE's
  `TriggerStrings.txt` has 39 lines like `=~Script Code"`.

**Semantic highlighting:**

| what                                | token type  | modifiers     |
|-------------------------------------|-------------|---------------|
| section name                        | `namespace` | `declaration` |
| key                                 | `property`  | `declaration` |
| `=`                                 | `operator`  |               |
| integer, real                       | `number`    |               |
| quoted string, unquoted text        | `string`    |               |
| `~Name` parameter                   | `parameter` |               |
| `WESTRING_*`, `TRIGSTR_*` reference | `variable`  | `readonly`    |
| comment                             | `comment`   |               |

Brackets and commas get no token.

## How it differs from the other parsers

BNI is a data format, not a programming language, and two of the shared
design rules apply in an adapted form:

1. **Its own scanner instead of a `LexSpec`.** Values are free text with
   spaces in them, and a character's meaning depends on its place in the
   line: `[` opens a section only at the start, and the first `=` splits
   key from value. The shared token lexer can't express that, and a
   `LexSpec` option for it would be a second lexer in disguise. So
   `lexer.rs` is a line scanner producing BNI's own token kinds
   (`token::Tok`: section name, key, value parts). It still uses the
   shared `Token` / `TokenStream`.

   The rule "one lexer for everything that colors source" holds. The
   parser, the semantic highlighter and the HTML viewer all read this
   scanner's tokens. The viewer gets them as semantic tokens through
   `parser_rs::render_html_tokens`.

2. **Parsing never fails as a whole.** Lines are independent, so a
   malformed line costs only itself. `parse` returns a `Document` (not a
   `Result`) whose `errors` list the malformed lines, and the rest of the
   file is still parsed and linted. The scanner never fails either: a
   line that fits no form becomes a `Junk` token for the parser to
   report.

## CLI

```sh
cargo run -- file.txt            # errors and lint warnings
cargo run -- --ast file.txt      # indented document tree
cargo run -- --dot file.txt      # Graphviz DOT
cargo run -- --html file.txt > file.ast.html && open file.ast.html
```

## Tests

```sh
cargo test
UPDATE_SNAPSHOTS=1 cargo test --test html_snapshot   # regenerate HTML goldens
```

## Library use

```rust
let doc = bni_rs::parse(&std::fs::read_to_string("TriggerData.txt")?);
for err in &doc.errors {
    eprintln!("{}:{}: {}", err.line, err.col, err.message);
}
if let Some(types) = doc.section("TriggerTypes") {
    for item in types.get("abilcode") {
        println!("{}", item.value_text()); // 0,1,1,WESTRING_TRIGTYPE_abilcode,integer
    }
}
```
