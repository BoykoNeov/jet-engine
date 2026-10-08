import re, sys
sys.stdout.reconfigure(encoding="utf-8")
V = r"W:\Claude_projects\jet engine\docs\visuals"
def t(n): return open(V + "\\" + n, encoding="utf-8").read()
READ = re.compile(r"\b(?:D\(\)|DATA|[IRd])\.([A-Za-z_][A-Za-z_0-9]*)")
cut = t("cutaway-template.html"); chart = t("template.html")
print("READ", sorted(set(READ.findall(cut))))
for name, src in (("cutaway", cut), ("chart", chart)):
    looked = sorted(set(m.group(2) for m in re.finditer(r"getElementById\((['\"])([A-Za-z0-9_-]+)\1\)", src)))
    decl = sorted(set(re.findall(r"""\bid=["']([A-Za-z0-9_-]+)["']""", src)))
    print("LOOKUP", name, len(looked), looked)
    print("DECL", name, len(decl), decl)
labels = re.search(r"const LOSS_LABEL = \{(.*?)\};", cut, re.S)
print("LOSS", sorted(set(re.findall(r"([A-Za-z_][A-Za-z_0-9]*):", labels.group(1)))))
m = re.search(r"const STN = \[(.*?)\];", cut, re.S)
print("STN", re.findall(r"\['([0-9]+)'", m.group(1)))
