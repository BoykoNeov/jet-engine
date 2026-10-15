$1!="shape" { k=$1" "$2" "$3; if ($6=="phi" && $10=="false") { v=$7+0;
   a=k" c"$5; if (!(a in lo) || v<lo[a]) lo[a]=v; if (!(a in hi) || v>hi[a]) hi[a]=v; nn[a]++;
   b=k" o"$4; if (!(b in lo) || v<lo[b]) lo[b]=v; if (!(b in hi) || v>hi[b]) hi[b]=v; nn[b]++ }
 if ($6=="inc") { v=$7+0; a=k" O"$4; if (!(a in lo) || v<lo[a]) lo[a]=v; if (!(a in hi) || v>hi[a]) hi[a]=v; nn[a]++;
   b=k" C"$5; if (!(b in lo) || v<lo[b]) lo[b]=v; if (!(b in hi) || v>hi[b]) hi[b]=v; nn[b]++ } }
END { for (a in lo) if (nn[a]>1) { split(a,s," "); t=substr(s[4],1,1); w=hi[a]-lo[a]; if (w>mx[t]) mx[t]=w; sum[t]+=w; cnt[t]++ }
  print "phi close-first runs: spread of set point over OPENING at fixed closing: max", mx["c"], "mean", sum["c"]/cnt["c"]
  print "phi close-first runs: spread over CLOSING at fixed opening: max", mx["o"], "mean", sum["o"]/cnt["o"]
  print "inc runs: spread over CLOSING at fixed opening: max", mx["O"], "; spread over OPENING at fixed closing: max", mx["C"], "mean", sum["C"]/cnt["C"] }
