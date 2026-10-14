import sys, re
W = sys.argv[1]
targets = {
 "src/fuel_transient.rs": ["r43_cut_below_ceiling", "try_clip_caps", "try_topping_required_below",
                            "r43_integrate_fuel", "integrate_fuel_lagged", "integrate_fuel_asym", "try_leg_below"],
 "src/demand_coordinate.rs": ["cap_free_gated", "r74_integrate_fuel_demand"],
 "src/lagged_bleed.rs": ["r65_integrate_fuel_valve_lag"],
 "src/two_lag.rs": ["r66_integrate_fuel_cascade"],
 "src/cross_loop.rs": ["r67_integrate_fuel_cross"],
 "src/three_loop.rs": ["r68_integrate_fuel_triple"],
 "src/cross_split.rs": ["r70_integrate_fuel_cross_triple"],
 "src/shared_actuator.rs": ["r72_integrate_fuel_shared"],
}
for f, names in targets.items():
    p = f"{W}/{f}"
    s = open(p, encoding="utf-8", newline="").read()
    for n in names:
        m = re.search(r"\bfn " + n + r"\b", s)
        assert m, (f, n)
        j = s.index("{\n", m.end())
        s = s[:j+2] + f'    let _zz = crate::fuel_transient::zz_ctx("{n}");\n' + s[j+2:]
    open(p, "w", encoding="utf-8", newline="").write(s)
p = f"{W}/src/fuel_transient.rs"
s = open(p, encoding="utf-8", newline="").read()
s = s.replace("/// CENSUS ONLY: log a reader's Err.", '''thread_local! { pub static ZZ_CTX: std::cell::RefCell<Vec<&'static str>> = const { std::cell::RefCell::new(Vec::new()) }; }
pub struct ZzCtx;
impl Drop for ZzCtx { fn drop(&mut self) { ZZ_CTX.with(|c| { c.borrow_mut().pop(); }); } }
pub fn zz_ctx(n: &'static str) -> ZzCtx { ZZ_CTX.with(|c| c.borrow_mut().push(n)); ZzCtx }
pub fn zz_ctx_str() -> String { ZZ_CTX.with(|c| c.borrow().join(">")) }

/// CENSUS ONLY: log a reader's Err.''', 1)
# add ctx + outcome to the hits log line
old = '{exe}\t{th}\t{mf:.9e}\t{}\n'
assert old in s, "fmt"
s = s.replace(old, '{exe}\t{th}\t{mf:.9e}\t{}\t{}\n')
s = s.replace('out.map_or("none".to_string(), |c| format!("{c:.9e}"))).as_bytes());',
              'out.map_or("none".to_string(), |c| format!("{c:.9e}")), zz_ctx_str()).as_bytes());', 1)
open(p, "w", encoding="utf-8", newline="").write(s)
print("tagged")
