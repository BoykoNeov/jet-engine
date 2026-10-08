import sys
for p in sys.argv[1:]:
    with open(p, "rb") as fh:
        b = fh.read()
    print(len(b), b.count(b"\r\n"), b.count(b"\n"), repr(b[-60:]), p.rsplit("/",1)[-1])
