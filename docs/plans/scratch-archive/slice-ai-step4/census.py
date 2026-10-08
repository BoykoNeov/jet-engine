import json
d = json.load(open(r"W:\temp\claude\slice-ai-step4\py.json"))
R = d["readers"]
def rows_of(name):
    r = R[name]["result"]
    if name == "arrest":
        return [x for a in r["arms"].values() for x in a["rows"]]
    return r.get("rows", [])
allrows = rows_of("liveness") + rows_of("arrest") + rows_of("saturation")
for k in ("arrested", "riding4_valid", "b_max_hit"):
    print(k, {v: sum(1 for x in allrows if x[k] == v) for v in (True, False)})
print("n_cut_fuel==0 rows", sum(1 for x in allrows if x["n_cut_fuel"] == 0), "of", len(allrows))
print("max_req_fuel<0", sum(1 for x in allrows if float.fromhex(x["max_req_fuel"]) < 0))
print("valve_moved==0", sum(1 for x in allrows if x["valve_moved"] == 0), "stator_moved==0", sum(1 for x in allrows if x["stator_moved"] == 0))
print("phi_air_built None", sum(1 for x in allrows if x["phi_air_built"] is None))
lv = R["liveness"]["result"]
print("liveness", {k: lv[k] for k in ("control_ok", "levers_woke", "fuel_off", "four_live", "n_split")})
ar = R["arrest"]["result"]
for a, v in ar["arms"].items():
    print("arrest", a, "marched", v["marched"], "arrested", v["arrested"], "monotone", v["monotone"])
print("control_bracket", ar["control_bracket"], "owner", ar["owner"])
sa = R["saturation"]["result"]
print("sat", {k: sa[k] for k in ("first_sat", "last_march", "cell", "impossible")})
for g in ("gains_clip", "gains_demand"):
    r = R[g]["result"]
    print(g, {k: r[k] for k in ("vacuous", "n_interior", "all_differenced", "ever_two_authorities", "control_nonzero", "max_mask_leak")})
    for a in r["arms"]:
        print("   arm", a["phi_air"], "riding", a["n_riding"], "sampled", a["n_sampled"], "interior", a["n_interior"], "skipped", a["skipped"], "auth", a["authority"], "masked", a["masked"], "zeros", a["zeros"], "maxcyc", a["max_cyc"], "leak", a["max_mask_leak"])
for n, v in R.items():
    print(n, "rig_after", v["rig_after"], "marches", len(v["marches"]), set((m["lag_coord"], m["phi_ref"]) for m in v["marches"]), "coords", set(m["coord"] for m in v["marches"]))
