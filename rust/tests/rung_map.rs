//! Roadmap A5 — the charts page's rung map is GENERATED from `CLAUDE.md`'s rung table and the
//! plain-language notes (`src/rung_map.rs`). These tests hold the generator to facts it does not
//! compute itself (a plain line count, a second scope sentence, hand-checked spot pins), and the
//! template to having no rung typed into it. `tests/visuals.rs::visuals_html_is_the_current_splice`
//! holds the BUILT page to the generator, so a new rung or a notes edit without a re-splice fails.

use std::path::{Path, PathBuf};

use turbojet::rung_map::{self, Rung};

fn repo() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("..") }

fn read(name: &str) -> String {
    let p = repo().join(name);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display())).replace("\r\n", "\n")
}

fn all() -> Vec<Rung> { rung_map::rungs(&read("CLAUDE.md"), &read("NOTES.md"), &rung_map::read_notes(&repo())) }

fn find<'a>(rs: &'a [Rung], num: &str) -> &'a Rung {
    rs.iter().find(|r| r.num == num).unwrap_or_else(|| panic!("rung {num} is not in the map"))
}

// ------------------------------------------------------------------- the list is CLAUDE.md's

/// Every table row reaches the map — counted by a plain line scan, not by the parser.
#[test]
fn every_table_row_is_in_the_map() {
    let rows = read("CLAUDE.md").lines().filter(|l| l.starts_with("| ") && l[2..].starts_with(|c: char| c.is_ascii_digit())).count();
    assert!(rows >= 109, "the plain scan found only {rows} rung rows — the scan went blind");
    assert_eq!(all().len(), rows, "the map and CLAUDE.md's rung table differ in length");
    let map = rung_map::read_repo(&repo());
    let in_families: usize = match map.at("families") {
        turbojet::visuals::Json::List(fs) => fs.iter().map(|f| match f.at("rungs") {
            turbojet::visuals::Json::List(r) => r.len(),
            _ => panic!("a family's rungs is not a list"),
        }).sum(),
        _ => panic!("families is not a list"),
    };
    assert_eq!(in_families, rows, "a rung is missing from, or repeated across, the families");
}

/// The top rung is the scope — read from CLAUDE.md's SECOND scope sentence ("**Current scope (rung
/// N).**"), which the generator does not read.
#[test]
fn the_top_rung_is_the_stated_scope() {
    let cm = read("CLAUDE.md");
    let key = "**Current scope (rung ";
    let at = cm.find(key).expect("CLAUDE.md has no **Current scope (rung N).** sentence") + key.len();
    let n: u32 = cm[at..].chars().take_while(char::is_ascii_digit).collect::<String>().parse().unwrap();
    assert_eq!(all().iter().map(|r| r.n).max(), Some(n));
    assert_eq!(rung_map::scope(&cm), n, "the two scope sentences of CLAUDE.md disagree");
}

/// Hand-checked against CLAUDE.md on 2026-11-15.
#[test]
fn spot_pins() {
    let rs = all();
    let order: Vec<&str> = rs.iter().map(|r| r.num.as_str()).collect();
    assert_eq!(&order[..4], ["1", "2", "2b", "3"], "2b sorts after 2");
    let pos = |n: &str| order.iter().position(|x| *x == n).unwrap();
    assert!(pos("72") + 1 == pos("73"), "72 comes before 73 (the table lists 73 first)");
    assert_eq!(find(&rs, "1").spec, "SPEC.md");
    assert_eq!(find(&rs, "2b").spec, "docs/rung2b-polytropic.md");
    assert_eq!(find(&rs, "3").spec, "docs/rung3-variable-cp.md");
    assert_eq!(find(&rs, "108").spec, "docs/rung108-spec.md");
    for (num, kind) in [("1", "cycle"), ("6", "cycle"), ("7", "diag"), ("30", "diag"), ("31", "run"), ("36", "diag"),
                        ("37", "run"), ("41", "diag"), ("45", "diag"), ("46", "run"), ("108", "run")] {
        assert_eq!(find(&rs, num).kind, kind, "rung {num}'s kind");
    }
    assert_eq!(find(&rs, "1").title, "The ideal Brayton cycle");
    assert_eq!(find(&rs, "73").title, "The applied reference", "a title in capitals is tamed");
    assert_eq!(find(&rs, "46").title, "The temperature limiter protects the wrong compressor", "a note's title wins");
    assert_eq!(find(&rs, "46").note.as_deref(), Some("docs/notes/46-52-fuel-limiters.md"));
    assert_eq!(find(&rs, "61").note.as_deref(), Some("docs/notes/53-56-61-airflow-levers.md"), "61 lives with the airflow levers");
    assert_eq!(find(&rs, "12").note.as_deref(), Some("NOTES.md"));
    assert_eq!(find(&rs, "17").note, None, "NOTES.md has no section for rung 17");
}

#[test]
fn the_families_cover_every_rung_once() {
    let cm = read("CLAUDE.md");
    let fams = rung_map::families(&cm);
    let ns: Vec<u32> = all().iter().map(|r| r.n).collect();
    for n in ns.iter() {
        let homes = fams.iter().filter(|f| f.members.contains(n)).count();
        assert_eq!(homes, 1, "rung {n} is in {homes} families");
    }
    for f in &fams {
        for m in &f.members {
            assert!(ns.contains(m), "family `{}` names rung {m}, which the table does not have", f.name);
        }
    }
    let airflow = fams.iter().find(|f| f.range == "53–56 + 61").expect("the `53–56 + 61` family");
    assert_eq!(airflow.members, [53, 54, 55, 56, 61]);
    let sched = fams.iter().find(|f| f.range == "57–60, 62–63").expect("the `57–60, 62–63` family");
    assert_eq!(sched.members, [57, 58, 59, 60, 62, 63]);
    assert_eq!(fams.iter().find(|f| f.range == "86–89").unwrap().name, "three levers, valve reference, slow & asymmetric valve",
               "a family name with commas stays whole");
}

// ---------------------------------------------------------------- the words are the notes'

/// Every `In one sentence` line in the notes reaches the page (counted by a plain scan), and rungs
/// 25–108 all read in plain words. A LATER rung without a note is allowed (it falls back to its
/// table hook); this pins the range written by 2026-11-15.
#[test]
fn the_notes_sentences_reach_the_map() {
    let notes = rung_map::read_notes(&repo());
    let lines = notes.iter().map(|(_, t)| t.lines().filter(|l| l.starts_with("**In one sentence:**")).count()).sum::<usize>();
    assert!(lines >= 68, "the plain scan found only {lines} sentences — the scan went blind");
    let rs = all();
    let titled = rs.iter().filter(|r| r.plain && r.note.as_deref().is_some_and(|p| !p.contains("69-84"))).count();
    assert_eq!(titled, lines, "a note's `In one sentence` did not reach its rung");
    let unplain: Vec<&str> = rs.iter().filter(|r| (25..=108).contains(&r.n) && !r.plain).map(|r| r.num.as_str()).collect();
    assert!(unplain.is_empty(), "rungs {unplain:?} fell back to the table hook");
    // The body of a sentence is the sentence (spot: rung 102's opening words, capitalised).
    assert!(find(&rs, "102").body.starts_with("A second burner behind the turbine"), "{}", find(&rs, "102").body);
    // A shared bullet keeps its label, so the reader sees it is shared.
    assert!(find(&rs, "74").body.starts_with("<b>Rungs 73–75</b>"), "{}", find(&rs, "74").body);
    // A rung's own bullet: `**Rung 69.** text` gives the text; `**Rung 76** found why` keeps its subject.
    assert!(find(&rs, "69").body.starts_with("Re-aim rung 68"), "{}", find(&rs, "69").body);
    assert!(find(&rs, "76").body.starts_with("<b>Rung 76</b> found why."), "{}", find(&rs, "76").body);
}

// ------------------------------------------------------------------------ the small parsers

#[test]
fn inline_markdown() {
    use rung_map::inline_md;
    assert_eq!(inline_md("a **b** c"), "a <b>b</b> c");
    assert_eq!(inline_md("an *italic* word"), "an <i>italic</i> word");
    assert_eq!(inline_md("`a<b` & `x*y*z`"), "<code>a&lt;b</code> &amp; <code>x*y*z</code>", "code is escaped, not styled");
    assert_eq!(inline_md("2 * 3"), "2 * 3", "an unmatched star stays");
    assert_eq!(inline_md("</script>"), "&lt;/script&gt;");
}

#[test]
fn hook_split() {
    use rung_map::split_hook;
    assert_eq!(split_hook("The **ideal** Brayton cycle: frozen."), ("The ideal Brayton cycle".into(), "Frozen.".into()));
    assert_eq!(split_hook("**Real components** — isentropic."), ("Real components".into(), "Isentropic.".into()));
    assert_eq!(split_hook("**THE SPLIT WALL** — rung 74's seam."), ("The split wall".into(), "Rung 74's seam.".into()));
    assert_eq!(split_hook("**Polytropic** knob (the asymmetry)."), ("Polytropic knob".into(), "(the asymmetry).".into()));
}

#[test]
#[should_panic(expected = "without exactly three cells")]
fn a_changed_table_shape_is_refused() {
    rung_map::table_rows("| Rung | Adds | Spec |\n|--|--|--|\n| 1 | a | `SPEC.md` | extra |\n");
}

#[test]
#[should_panic(expected = "no **Families** line")]
fn a_missing_families_line_is_refused() {
    rung_map::families("nothing here");
}

// ------------------------------------------------------------------------------ the template

fn template() -> String { read("docs/visuals/template.html") }

/// The template types no rung: the hand-kept array and its prose are gone, and the counts render.
#[test]
fn the_template_types_no_rung() {
    let t = template();
    for typed in ["const RUNGS", "docs/rung29-spec.md", "The shifting turbine", "first 29", "29 rungs", "84 cumulative"] {
        assert!(!t.contains(typed), "template.html types `{typed}` again — the rung map is generated (src/rung_map.rs)");
    }
    assert_eq!(t.matches("class=\"rung-scope\"").count(), 2, "the hero line and the heading render the rung count");
    assert!(t.contains("RUNG_MAP.scope") && t.contains("RUNG_MAP.families"), "the script no longer renders the map");
    assert!(t.contains("id=\"rung-list\"") && t.contains("id=\"rungs\""), "the map's ids are kept");
}

/// The built page carries the map, and no `</` inside it (which could close the script early).
#[test]
fn the_built_page_embeds_the_map_safely() {
    let page = read("docs/visuals/turbojet-visuals.html");
    let at = page.find("const RUNG_MAP = ").expect("the built page has no RUNG_MAP") + "const RUNG_MAP = ".len();
    let line = &page[at..at + page[at..].find('\n').unwrap()];
    assert!(line.starts_with('{') && line.ends_with("};"), "RUNG_MAP is not one JSON object on its line");
    assert!(!line.contains("</"), "the embedded map contains `</`");
    assert!(line.contains("\"num\":\"108\""), "the built page's map stops short of rung 108 — re-splice");
}
