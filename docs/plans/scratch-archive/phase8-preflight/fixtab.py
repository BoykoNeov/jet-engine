p = r"W:\Claude_projects\jet engine\docs\plans\todo-rust-port.md"
t = open(p, encoding="utf-8", newline="").read()
bad = "W:\t" + "emp"
n = t.count(bad)
t = t.replace(bad, "W:\temp")
open(p, "w", encoding="utf-8", newline="").write(t)
print("fixed", n)
