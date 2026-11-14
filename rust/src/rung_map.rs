//! The charts page's **rung map**, generated (roadmap A5, the larger option).
//!
//! Until 2026-11-15 `docs/visuals/template.html` carried a hand-kept array of the first 29 rungs and
//! fell 79 rungs behind. This module builds the map from the two places the project already keeps
//! the rungs, so a new rung reaches the page by a re-splice and never by hand:
//!
//! * **`CLAUDE.md`'s rung table** — the list itself: every rung's number, its spec, a title and its
//!   one-line hook; the **families** line (how the rungs group); the **read-only diagnostics** line
//!   (which rungs only read the engine); and the "current scope" sentence.
//! * **the plain-language notes** — `docs/notes/*.md` (`## Rung N — title` then
//!   `**In one sentence:** …`, or rungs 69–84's `- **Rung N.** …` bullets) and the root `NOTES.md`
//!   (`# Rung N — …` headings, rungs 1–22). Where a rung has a sentence, the page shows it and its
//!   title instead of the table's hook, which is written for the build assistant, not a reader.
//!
//! [`build`] is pure (text in, JSON out); [`read_repo`] reads the files for the CLI and the tests.
//! Every parse that meets a shape it does not know PANICS with the line it was reading: a silent
//! empty map is the one failure the staleness gate (`tests/visuals.rs`) could not see.

use std::path::Path;

use crate::visuals::Json;

/// One rung as the page shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct Rung {
    /// As the table writes it: `"1"`, `"2b"`, `"108"`.
    pub num: String,
    /// The integer part, for sorting and family lookup (`"2b"` → 2).
    pub n: u32,
    /// HTML (escaped, with `<b>`/`<i>`/`<code>` from the Markdown).
    pub title: String,
    /// HTML.
    pub body: String,
    /// `"cycle"`, `"diag"` (read-only) or `"run"` (an operating point).
    pub kind: &'static str,
    /// The table's third column, e.g. `docs/rung46-spec.md`.
    pub spec: String,
    /// Where the plain-language story is, e.g. `docs/notes/46-52-fuel-limiters.md`, if anywhere.
    pub note: Option<String>,
    /// True when `body` is a plain-language sentence from the notes, false when it is the table's hook.
    pub plain: bool,
}

/// One family from the families line.
#[derive(Clone, Debug, PartialEq)]
pub struct Family {
    /// The ranges as written, e.g. `"53–56 + 61"`.
    pub range: String,
    /// The name as written, e.g. `"airflow levers (steady)"`.
    pub name: String,
    /// The rung numbers it covers.
    pub members: Vec<u32>,
}

// ------------------------------------------------------------------------------- small helpers

/// `"57–60, 62–63"` style lists: integers and en-dash (or hyphen) ranges, separated by `,` or `+`.
/// Returns `None` if `s` is not entirely such a list.
fn parse_ranges(s: &str) -> Option<Vec<u32>> {
    let mut out = Vec::new();
    for part in s.split([',', '+']) {
        let part = part.trim();
        if part.is_empty() {
            return None;
        }
        let mut ends = part.splitn(2, ['–', '-']);
        let a: u32 = ends.next()?.trim().parse().ok()?;
        let b: u32 = match ends.next() {
            Some(b) => b.trim().parse().ok()?,
            None => a,
        };
        if b < a {
            return None;
        }
        out.extend(a..=b);
    }
    Some(out)
}

/// The integer part of a rung label (`"2b"` → 2).
fn int_part(num: &str) -> u32 {
    let digits: String = num.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().unwrap_or_else(|_| panic!("rung map: `{num}` is not a rung number"))
}

/// Escape for HTML text.
fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// The inline Markdown the table and the notes use, as HTML: `**b**`, `*i*`, `` `code` ``.
/// Everything else is escaped text. An unmatched marker is kept literally.
pub fn inline_md(s: &str) -> String {
    let s = esc(s);
    let s = pair(&s, "`", "<code>", "</code>");
    let s = pair(&s, "**", "<b>", "</b>");
    pair(&s, "*", "<i>", "</i>")
}

/// Replace each matched pair of `mark` with `open`/`close`; a trailing unmatched mark stays.
/// Text already inside `<code>…</code>` is left alone, so `a*b` in code is not italicised.
fn pair(s: &str, mark: &str, open: &str, close: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    loop {
        // Copy any <code> span verbatim.
        let next_mark = rest.find(mark);
        let next_code = if mark == "`" { None } else { rest.find("<code>") };
        match (next_mark, next_code) {
            (Some(m), Some(c)) if c < m => {
                let end = rest[c..].find("</code>").map_or(rest.len(), |e| c + e + "</code>".len());
                out.push_str(&rest[..end]);
                rest = &rest[end..];
            }
            (Some(m), _) => {
                let after = &rest[m + mark.len()..];
                match after.find(mark) {
                    Some(e) if e > 0 => {
                        out.push_str(&rest[..m]);
                        out.push_str(open);
                        out.push_str(&after[..e]);
                        out.push_str(close);
                        rest = &after[e + mark.len()..];
                    }
                    _ => {
                        out.push_str(&rest[..m + mark.len()]);
                        rest = after;
                    }
                }
            }
            (None, _) => {
                out.push_str(rest);
                return out;
            }
        }
    }
}

/// Strip the Markdown markers (for a plain-text title test).
fn plain(s: &str) -> String {
    s.replace("**", "").replace('`', "")
}

/// Upper-case the first letter.
fn cap(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// `"THE APPLIED REFERENCE"` → `"The applied reference"`; mixed case is left alone.
fn tame_caps(s: &str) -> String {
    if s.chars().any(|c| c.is_lowercase()) {
        s.to_string()
    } else {
        cap(&s.to_lowercase())
    }
}

/// Lines joined into one paragraph (continuation lines are indented in the notes).
fn join_lines<'a>(lines: impl Iterator<Item = &'a str>) -> String {
    lines.map(str::trim).collect::<Vec<_>>().join(" ")
}

// ------------------------------------------------------------------------------- CLAUDE.md

/// The rung table's rows: `(num, hook-markdown, spec)`, in the table's order.
pub fn table_rows(claude_md: &str) -> Vec<(String, String, String)> {
    let mut rows = Vec::new();
    for line in claude_md.lines() {
        let t = line.trim();
        if !t.starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = t.trim_matches('|').split('|').map(str::trim).collect();
        let Some(first) = cells.first() else { continue };
        if !first.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            continue; // the header row and the |---| rule
        }
        // The hook may itself contain `|`? It does not today; a row with other than three cells
        // is a shape this parser does not know.
        assert!(cells.len() == 3, "rung map: a rung-table row without exactly three cells: {t}");
        let spec = cells[2].trim_matches('`').to_string();
        assert!(spec.ends_with(".md"), "rung map: rung {} names no spec file: {t}", cells[0]);
        rows.push((cells[0].to_string(), cells[1].to_string(), spec));
    }
    assert!(!rows.is_empty(), "rung map: no rung-table rows found in CLAUDE.md");
    rows
}

/// The `**Families** … · …` paragraph, parsed.
pub fn families(claude_md: &str) -> Vec<Family> {
    let start = claude_md.find("**Families**").expect("rung map: CLAUDE.md has no **Families** line");
    let after = &claude_md[start..];
    let colon = after.find(':').expect("rung map: the **Families** line has no colon");
    let end = after.find("**Read-only diagnostics").expect("rung map: no **Read-only diagnostics** after the families");
    let text = join_lines(after[colon + 1..end].lines());
    let text = text.trim().trim_end_matches('.');
    let mut out = Vec::new();
    for item in text.split(" · ") {
        let item = item.trim();
        // The longest prefix that is a range list, ending at a word boundary before the name.
        let words: Vec<&str> = item.split(' ').collect();
        let mut best = None;
        for k in 1..words.len() {
            let head = words[..k].join(" ");
            if let Some(m) = parse_ranges(&head) {
                best = Some((k, head, m));
            }
        }
        let (k, range, members) = best.unwrap_or_else(|| panic!("rung map: family `{item}` starts with no rung range"));
        out.push(Family { range, name: words[k..].join(" "), members });
    }
    out
}

/// The `**Read-only diagnostics: 7–30, 36, …**` list.
pub fn read_only(claude_md: &str) -> Vec<u32> {
    let key = "**Read-only diagnostics:";
    let start = claude_md.find(key).expect("rung map: no **Read-only diagnostics:** in CLAUDE.md") + key.len();
    let end = claude_md[start..].find("**").expect("rung map: the read-only list is not closed") + start;
    parse_ranges(&claude_md[start..end]).unwrap_or_else(|| panic!("rung map: cannot read the read-only list `{}`", &claude_md[start..end]))
}

/// The `the current scope is **rung N**` number.
pub fn scope(claude_md: &str) -> u32 {
    let key = "the current scope is **rung ";
    let start = claude_md.find(key).expect("rung map: no `the current scope is **rung N**` in CLAUDE.md") + key.len();
    let digits: String = claude_md[start..].chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().expect("rung map: the current scope is not a number")
}

/// A table hook split into a title and the rest: the text before the first ` — `, `: ` or ` (`,
/// whichever comes first. A title in capitals is tamed to sentence case.
pub fn split_hook(hook: &str) -> (String, String) {
    let cut = [" — ", ": ", " ("]
        .iter()
        .filter_map(|sep| hook.find(sep).map(|i| (i, *sep)))
        .min_by_key(|&(i, _)| i);
    let (title, rest) = match cut {
        Some((i, " (")) => (&hook[..i], hook[i + 1..].to_string()),
        Some((i, sep)) => (&hook[..i], hook[i + sep.len()..].to_string()),
        None => (hook, String::new()),
    };
    let title = plain(title);
    (tame_caps(title.trim()), cap(rest.trim()))
}

// --------------------------------------------------------------------------------- the notes

/// One rung's plain-language entry in a notes file.
#[derive(Clone, Debug, PartialEq)]
pub struct Sentence {
    pub n: u32,
    /// The section heading's title (`## Rung N — title`), for an `In one sentence` entry.
    pub title: Option<String>,
    /// The sentence, Markdown.
    pub text: String,
    /// Which wins when a rung has several: 0 its own `In one sentence`, 1 its own bullet, 2 a
    /// bullet shared by several rungs.
    pub rank: u8,
}

/// The `In one sentence` entries (and 69–84's own bullets) in one notes file.
pub fn note_sentences(text: &str) -> Vec<Sentence> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out: Vec<Sentence> = Vec::new();
    let mut heading: Option<(u32, String)> = None;
    let mut i = 0;
    while i < lines.len() {
        let l = lines[i];
        if l.starts_with('#') {
            let h = l.trim_start_matches('#').trim();
            heading = h.strip_prefix("Rung ").and_then(|r| {
                let (n, title) = r.split_once(" — ")?;
                Some((n.trim().parse().ok()?, title.trim().to_string()))
            });
        } else if let Some(s) = l.strip_prefix("**In one sentence:**") {
            let (n, title) = heading.clone().unwrap_or_else(|| panic!("rung map: an `In one sentence` line under no `Rung N — title` heading: {l}"));
            let mut para = vec![s];
            while i + 1 < lines.len() && !lines[i + 1].trim().is_empty() {
                i += 1;
                para.push(lines[i]);
            }
            if !out.iter().any(|s| s.n == n && s.rank == 0) {
                out.push(Sentence { n, title: Some(title), text: join_lines(para.into_iter()), rank: 0 });
            }
        } else if let Some(r) = l.strip_prefix("- **Rung ") {
            // `- **Rung 69.** text` — a bullet that is one rung's own; or `- **Rung 76** found why.`,
            // where the label runs into the sentence and so is kept as its subject.
            let digits: String = r.chars().take_while(char::is_ascii_digit).collect();
            let tail = &r[digits.len()..];
            let s = if let Some(s) = tail.strip_prefix(".** ") {
                Some(s)
            } else if tail.starts_with("** ") {
                Some(&l[2..])
            } else {
                None
            };
            if let (Ok(n), Some(s)) = (digits.parse::<u32>(), s) {
                {
                    let mut para = vec![s];
                    while i + 1 < lines.len() && lines[i + 1].starts_with("  ") && !lines[i + 1].trim().is_empty() {
                        i += 1;
                        para.push(lines[i]);
                    }
                    out.push(Sentence { n, title: None, text: join_lines(para.into_iter()), rank: 1 });
                }
            }
        } else if let Some(r) = l.strip_prefix("- **Rungs ") {
            // `- **Rungs 73–75** text` — one bullet for several rungs. Each of them gets the whole
            // bullet, its bold "Rungs 73–75" label kept so the reader sees it is shared. A rung's
            // own entry, if any, wins (see the sort in `rungs`).
            if let Some((head, _)) = r.split_once("**") {
                if let Some(members) = parse_ranges(head.trim_end_matches('.')) {
                    let mut para = vec![&l[2..]];
                    while i + 1 < lines.len() && lines[i + 1].starts_with("  ") && !lines[i + 1].trim().is_empty() {
                        i += 1;
                        para.push(lines[i]);
                    }
                    let text = join_lines(para.into_iter());
                    for n in members {
                        out.push(Sentence { n, title: None, text: text.clone(), rank: 2 });
                    }
                }
            }
        }
        i += 1;
    }
    out
}

/// The rungs a notes file's NAME covers: `46-52-fuel-limiters` → 46..=52; `53-56-61-…` → 53..=56, 61.
fn file_covers(stem: &str) -> Vec<u32> {
    let nums: Vec<u32> = stem.split('-').map_while(|p| p.parse().ok()).collect();
    match nums.as_slice() {
        [] => Vec::new(),
        [a] => vec![*a],
        [a, b, rest @ ..] => (*a..=*b).chain(rest.iter().copied()).collect(),
    }
}

/// The rungs `NOTES.md` (rungs 1–22) has a section for: its `# Rung N — …` headings, and rung 1,
/// which is the file's opening rather than a heading of its own.
pub fn notes_md_rungs(notes_md: &str) -> Vec<String> {
    let mut out = vec!["1".to_string()];
    for l in notes_md.lines() {
        if let Some(r) = l.strip_prefix("# Rung ") {
            if let Some((n, _)) = r.split_once(" — ") {
                out.push(n.trim().to_string());
            }
        }
    }
    out
}

// ------------------------------------------------------------------------------------ build

/// The map: every table row, sorted by number, with its family, kind, title, body and pointers.
/// `notes` is `(repo-relative path, text)` per `docs/notes/*.md` file (README excluded), in any order.
pub fn rungs(claude_md: &str, notes_md: &str, notes: &[(String, String)]) -> Vec<Rung> {
    let claude_md = claude_md.replace("\r\n", "\n");
    let notes_md = notes_md.replace("\r\n", "\n");
    let fams = families(&claude_md);
    let diag = read_only(&claude_md);
    let notes_md_has = notes_md_rungs(&notes_md);
    let mut sentences: Vec<(Sentence, String)> = Vec::new();
    let mut covers: Vec<(Vec<u32>, String)> = Vec::new();
    let mut sorted_notes: Vec<&(String, String)> = notes.iter().collect();
    sorted_notes.sort_by(|a, b| a.0.cmp(&b.0));
    for (path, text) in sorted_notes {
        let text = text.replace("\r\n", "\n");
        let stem = Path::new(path).file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let covered = file_covers(stem);
        // A titled `In one sentence` is always its rung's own; a `- **Rung N.**` bullet counts only
        // in the file whose name covers N (another family's file may cite rung N in a bullet).
        for s in note_sentences(&text) {
            if s.rank == 0 || covered.contains(&s.n) {
                sentences.push((s, path.clone()));
            }
        }
        covers.push((covered, path.clone()));
    }
    // By rank, so a rung's own `In one sentence` wins over its bullet, and both over a shared one.
    sentences.sort_by_key(|(s, _)| s.rank);
    let cycle: Vec<u32> = fams.iter().filter(|f| f.name == "cycle").flat_map(|f| f.members.clone()).collect();
    let mut out: Vec<Rung> = table_rows(&claude_md)
        .into_iter()
        .map(|(num, hook, spec)| {
            let n = int_part(&num);
            let kind = if cycle.contains(&n) {
                "cycle"
            } else if diag.contains(&n) {
                "diag"
            } else {
                "run"
            };
            let (t_title, t_body) = split_hook(&hook);
            // A sentence belongs to rung `n` only for the plain integer label (not `2b`).
            let own = (num == n.to_string()).then(|| sentences.iter().find(|(s, _)| s.n == n)).flatten();
            let (title, body, plain_text, note) = match own {
                Some((s, path)) => (
                    s.title.as_deref().map_or_else(|| inline_md(&t_title), inline_md),
                    inline_md(&cap(s.text.trim())),
                    true,
                    Some(path.clone()),
                ),
                None => {
                    let note = if notes_md_has.contains(&num) {
                        Some("NOTES.md".to_string())
                    } else {
                        covers.iter().find(|(c, _)| c.contains(&n)).map(|(_, p)| p.clone())
                    };
                    (inline_md(&t_title), inline_md(&t_body), false, note)
                }
            };
            Rung { num, n, title, body, kind, spec, note, plain: plain_text }
        })
        .collect();
    // Sort by number; `2b` after `2` (labels of equal integer part by their own text).
    out.sort_by(|a, b| a.n.cmp(&b.n).then(a.num.len().cmp(&b.num.len())).then(a.num.cmp(&b.num)));
    out
}

/// The JSON the template's `RUNG_MAP` receives:
/// `{"scope": N, "count": rows, "families": [{"range", "name", "rungs": [{num,title,body,kind,spec,note}]}]}`.
/// Families appear in order of their first rung; a rung no family names goes in a last family
/// named `"not yet in a family"` (none today — `tests/visuals.rs` pins that).
pub fn build(claude_md: &str, notes_md: &str, notes: &[(String, String)]) -> Json {
    let all = rungs(claude_md, notes_md, notes);
    let cm = claude_md.replace("\r\n", "\n");
    let mut fams = families(&cm);
    fams.sort_by_key(|f| f.members.iter().min().copied().unwrap_or(0));
    let rung_json = |r: &Rung| {
        Json::Obj(vec![
            ("num".into(), r.num.as_str().into()),
            ("title".into(), r.title.as_str().into()),
            ("body".into(), r.body.as_str().into()),
            ("kind".into(), r.kind.into()),
            ("spec".into(), r.spec.as_str().into()),
            ("note".into(), r.note.as_deref().map_or(Json::Null, Json::from)),
        ])
    };
    let mut groups: Vec<Json> = Vec::new();
    for f in &fams {
        let members: Vec<Json> = all.iter().filter(|r| f.members.contains(&r.n)).map(rung_json).collect();
        groups.push(Json::Obj(vec![
            ("range".into(), f.range.as_str().into()),
            ("name".into(), f.name.as_str().into()),
            ("rungs".into(), Json::List(members)),
        ]));
    }
    let orphans: Vec<Json> =
        all.iter().filter(|r| !fams.iter().any(|f| f.members.contains(&r.n))).map(rung_json).collect();
    if !orphans.is_empty() {
        groups.push(Json::Obj(vec![
            ("range".into(), "".into()),
            ("name".into(), "not yet in a family".into()),
            ("rungs".into(), Json::List(orphans)),
        ]));
    }
    Json::Obj(vec![
        ("scope".into(), Json::Int(scope(&cm) as i64)),
        ("count".into(), Json::Int(all.len() as i64)),
        ("families".into(), Json::List(groups)),
    ])
}

/// The notes files as [`build`] takes them: every `docs/notes/*.md` but the README, keyed by its
/// repo-relative path with forward slashes.
pub fn read_notes(repo: &Path) -> Vec<(String, String)> {
    let dir = repo.join("docs").join("notes");
    let mut out = Vec::new();
    for e in std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("cannot list {}: {e}", dir.display())) {
        let p = e.expect("notes dir entry").path();
        let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
        if name.ends_with(".md") && name != "README.md" {
            let text = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display()));
            out.push((format!("docs/notes/{name}"), text));
        }
    }
    out.sort();
    out
}

/// [`build`] on the repository's own files (`repo` = the repository root).
pub fn read_repo(repo: &Path) -> Json {
    let read = |p: &Path| std::fs::read_to_string(p).unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display()));
    build(&read(&repo.join("CLAUDE.md")), &read(&repo.join("NOTES.md")), &read_notes(repo))
}
