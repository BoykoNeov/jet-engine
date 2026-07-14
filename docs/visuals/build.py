# Splice data.json into template.html -> turbojet-visuals.html (the finished page).
# Regenerate data first with extract_data.py if the model changed.
from pathlib import Path

HERE = Path(__file__).resolve().parent
tpl = (HERE / "template.html").read_text(encoding="utf-8")
data = (HERE / "data.json").read_text(encoding="utf-8")
assert "/*__DATA_JSON__*/" in tpl, "template placeholder missing"
out = tpl.replace("/*__DATA_JSON__*/", data, 1)
(HERE / "turbojet-visuals.html").write_text(out, encoding="utf-8")
print(f"built turbojet-visuals.html ({len(out):,} bytes)")
