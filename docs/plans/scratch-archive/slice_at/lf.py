import sys
for p in sys.argv[1:]:
    b = open(p, "rb").read()
    n = b.count(b"\r\n")
    open(p, "wb").write(b.replace(b"\r\n", b"\n"))
    print(p.rsplit("/", 1)[-1], "crlf fixed:", n)
