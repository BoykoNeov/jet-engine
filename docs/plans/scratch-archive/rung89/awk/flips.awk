$1!="shape" && $6=="phi" { R[$1" "$2" "$3" "$4" "$5]=$19; c[$1" "$2" "$3]=1 }
END {
 split("0.01 0.02 0.05 0.1 0.2",T," "); bad=0; lines=0
 for (cc in c) for (i=1;i<=5;i++) {
   r=""; k=""
   for (j=1;j<=5;j++) { r=r (R[cc" "T[i]" "T[j]]>1?"+":"-"); k=k (R[cc" "T[j]" "T[i]]>1?"+":"-") }
   lines+=2
   if (r !~ /^\+*-*$/) { bad++; print "ROW", cc, "to="T[i], r }
   if (k !~ /^\+*-*$/) { bad++; print "COL", cc, "tc="T[i], k }
 }
 print "lines checked", lines, "with more than one sign change or a -+ step:", bad
}
