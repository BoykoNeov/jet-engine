# Census patch: force `below_ceiling` ON everywhere and log every ceiling search.
# Applied to a throwaway worktree only. Usage: python force_on.py <worktree>/rust/src
import sys

R = sys.argv[1]


def sub(f, old, new, count=1):
    p = f"{R}/{f}"
    s = open(p, encoding="utf-8", newline="").read()
    n = s.count(old)
    assert n == count, (f, old[:70], n)
    s = s.replace(old, new)
    open(p, "w", encoding="utf-8", newline="").write(s)


# 1. rung 43's three reads of the limiters' switch -> forced on
sub("fuel_transient.rs", "lim.below_ceiling", "true", 3)
# 2. the rung-65+ single source -> forced on
sub("lagged_bleed.rs", "    lim.below_ceiling\n}", "    true\n}")
# 3. the core's carrier defaults ON (stator route, rung 74 cells, readers)
sub("two_spool_transient.rs", "below_ceiling: Cell::new(false),", "below_ceiling: Cell::new(true),")
# 4. log every ceiling search: exe, test thread, mf_sched, outcome
sub("fuel_transient.rs",
    """    pub fn try_solvable_ceiling(
        &self, flight: &FlightCondition, nu_lp: f64, nu_hp: f64, mf: f64,
    ) -> Option<f64> {
""",
    """    pub fn try_solvable_ceiling(
        &self, flight: &FlightCondition, nu_lp: f64, nu_hp: f64, mf: f64,
    ) -> Option<f64> {
        let out = self.zz_ceiling(flight, nu_lp, nu_hp, mf);
        {
            use std::io::Write;
            let exe = std::env::current_exe().ok()
                .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
                .unwrap_or_default();
            let th = std::thread::current().name().unwrap_or("?").to_string();
            if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true)
                .open(r"W:\\temp\\claude\\ceiling-rollout\\hits.tsv") {
                let _ = f.write_all(format!("{exe}\\t{th}\\t{mf:.9e}\\t{}\\n",
                    out.map_or("none".to_string(), |c| format!("{c:.9e}"))).as_bytes());
            }
        }
        out
    }

    fn zz_ceiling(
        &self, flight: &FlightCondition, nu_lp: f64, nu_hp: f64, mf: f64,
    ) -> Option<f64> {
""")
print("forced on")
