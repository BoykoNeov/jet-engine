import sys
a=[l.rstrip("\n") for l in open(sys.argv[1],encoding="utf-8")]
b=[l.rstrip("\n") for l in open(sys.argv[2],encoding="utf-8") if not l.startswith("LAW ")]
print("lines",len(a),len(b))
n=0
for i,(x,y) in enumerate(zip(a,b)):
    if x!=y:
        n+=1; print("-",x); print("+",y)
print("differing lines",n)
