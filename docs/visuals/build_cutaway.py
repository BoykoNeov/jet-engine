"""Splice data.json into cutaway-template.html -> turbojet-cutaway.html.

The cutaway page needs only the design point and the ideal / real station
tables, so it embeds a trimmed copy of data.json (regenerate that with
extract_data.py). Run from anywhere: paths are relative to this file.
"""
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
KEEP = ["stations", "V0", "V9", "M9", "T9", "specific_thrust", "tsfc",
        "eta_thermal", "eta_propulsive", "eta_overall", "points", "legs"]


def main():
    d = json.loads((HERE / "data.json").read_text(encoding="utf-8"))
    data = {"design": d["design"],
            "ideal": {k: d["ideal"][k] for k in KEEP},
            "real": {k: d["real"][k] for k in KEEP}}
    tpl = (HERE / "cutaway-template.html").read_text(encoding="utf-8")
    assert tpl.count("/*__DATA__*/") == 1
    html = tpl.replace("/*__DATA__*/", json.dumps(data, separators=(",", ":")))
    out = HERE / "turbojet-cutaway.html"
    out.write_text(html, encoding="utf-8", newline="\n")
    print(f"{len(html)} bytes -> {out}")


if __name__ == "__main__":
    main()
