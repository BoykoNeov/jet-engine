// Rung 104 probe P21 — the CLI's Python-era panels under the default law: which still equal their Python segment
// (re-cuts ignored: a re-cut panel is reported, not judged), which differ, which panic.
use turbojet::panels::{Design, PANELS, RUST_OWNED};
use turbojet::pyfmt::Printer;

fn main() {
    let golden = std::fs::read_to_string("W:/Claude_projects/jet engine/rust/oracle/main_stdout.txt").unwrap();
    let segs: Vec<(String, usize, usize)> = std::fs::read_to_string("W:/Claude_projects/jet engine/rust/oracle/main_segments.tsv")
        .unwrap().lines().filter(|l| !l.starts_with('#'))
        .map(|l| { let f: Vec<&str> = l.split('\t').collect(); (f[0].to_string(), f[1].parse().unwrap(), f[2].parse().unwrap()) })
        .collect();
    let python: Vec<_> = PANELS.iter().filter(|(n, _)| !RUST_OWNED.contains(n)).collect();
    let design = Design::new();
    std::panic::set_hook(Box::new(|_| {}));
    for (i, ((name, panel), seg)) in python.into_iter().zip(&segs).enumerate() {
        let want = &golden[seg.1..seg.2];
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { let mut p = Printer::new(); panel(&mut p, &design); p.as_str().to_string() }));
        match r {
            Ok(got) if got == want => {}
            Ok(got) => {
                let k = want.lines().zip(got.lines()).position(|(a, b)| a != b);
                println!("step {i:2} {name}: DIFFERS at line {k:?}");
            }
            Err(e) => println!("step {i:2} {name}: PANICS {}", e.downcast_ref::<String>().map(|s| s.chars().take(100).collect::<String>()).unwrap_or_default()),
        }
    }
    for name in RUST_OWNED {
        let (_, panel) = PANELS.iter().find(|(n, _)| n == name).unwrap();
        let want = std::fs::read_to_string(format!("W:/Claude_projects/jet engine/rust/oracle/rust_owned/{name}.txt")).unwrap();
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { let mut p = Printer::new(); panel(&mut p, &design); p.as_str().to_string() }));
        match r {
            Ok(got) if got == want => {}
            Ok(_) => println!("rust-owned {name}: DIFFERS"),
            Err(_) => println!("rust-owned {name}: PANICS"),
        }
    }
}
