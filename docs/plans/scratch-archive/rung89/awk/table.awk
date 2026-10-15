$1!="shape" && $6=="phi" { R[$1" "$2" "$3" "$4" "$5]=$19; S[$1" "$2" "$3" "$4" "$5]=$7; c[$1" "$2" "$3]=1 }
END {
 split("0.01 0.02 0.05 0.1 0.2",T," ")
 for (cc in c) { split(cc,p," "); line="| "p[1]" | "p[2]" | "p[3]" |"
   d="—"; for (i=1;i<=5;i++) if (R[cc" "T[i]" "T[i]]>1) d=T[i]; line=line" "d" |"
   for (j=1;j<=5;j++) { s="—"; for (i=1;i<=5;i++) if (R[cc" "T[i]" "T[j]]>1) s=T[i]; line=line" "s" |" }
   print line }
}
