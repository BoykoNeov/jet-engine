import json, glob, collections, sys, io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
D = r"W:/temp/claude/slice-aj-preflight/out"
for run in ("suite_ctl", "suite_mut"):
    fs = sorted(glob.glob(f"{D}/{run}.*.json"))
    print(f"\n######## {run}: {len(fs)} process files")
    agg = collections.defaultdict(collections.Counter); scal = collections.Counter(); ctl = []
    for f in fs:
        J = json.load(open(f, encoding="utf-8"))
        ctl.append((f.split(".")[-3], J["control"]))
        for k, v in J.items():
            if isinstance(v, dict) and k not in ("control",):
                agg[k].update(v)
            elif isinstance(v, int) and k != "pid":
                scal[k] += v
    for w, c in ctl: print("  control", w, c)
    print("  scalars", dict(scal))
    for k in ("rig_built", "rig_of_rig", "r81p_on_rig", "inc_args", "phi_ref_at_cap", "nest_where"):
        print(f"  --- {k} ({sum(agg[k].values())})")
        for kk, v in agg[k].most_common(40): print(f"     {v:>9}  {kk}")
    print(f"  --- r81p_on_top: {len(agg['r81p_on_top'])} distinct, {sum(agg['r81p_on_top'].values())} calls")
    for kk, v in sorted(agg["r81p_on_top"].items()): print(f"     {v:>9}  {kk}")
    if agg["calls"]:
        calls = agg["calls"]
        by_def = collections.Counter()
        for kk in calls:
            d, n, rc = kk.split(" | ")
            by_def[(d, n)] += calls[kk]
        print(f"  --- calls: {len(calls)} (definer,name,recv) keys; {len(by_def)} (definer,name) reached")
        json.dump(sorted(calls.items()), open(f"{D}/census_calls.json", "w"), indent=0)
