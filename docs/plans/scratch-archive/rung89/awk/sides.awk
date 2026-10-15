$1!="shape" && $6=="phi" { R[$1" "$2" "$3" "$4" "$5]=$19; c[$1" "$2" "$3]=1 }
END {
 split("0.01 0.02 0.05 0.1 0.2",T," "); dp=0;c1=0;c5=0; stronger=0; eq=0; weaker=0; nr=0; nc=0
 for (cc in c) { rp=0; cp=0
  for (i=1;i<=5;i++) { if (R[cc" "T[i]" "T[i]]>1) dp++; if (R[cc" "T[i]" 0.01"]>1) c1++; if (R[cc" "T[i]" 0.2"]>1) c5++
    if (R[cc" 0.01 "T[i]]>1) rp++; if (R[cc" "T[i]" 0.01"]>1) cp++
    for (j=2;j<=5;j++) { if (R[cc" "T[i]" "T[j]] >= R[cc" "T[i]" "T[j-1]]) { nr++; print "row up:", cc, "to="T[i], "tc="T[j] }
                         if (R[cc" "T[j]" "T[i]] >= R[cc" "T[j-1]" "T[i]]) { nc++; print "col up:", cc, "tc="T[i], "to="T[j] } } }
  if (rp<cp) stronger++; else if (rp==cp) eq++; else weaker++ }
 print "diag inc-cheaper", dp, "/95; col tc=0.01:", c1, "/95; col tc=0.2:", c5, "/95; closing flips sooner in", stronger, "equal", eq, "later", weaker, "; rows not decreasing", nr, "cols not decreasing", nc
}
