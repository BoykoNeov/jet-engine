$1!="shape" { k=$1" "$2" "$3" "$4" "$5" "$6; S[k]=$7; A[k]=$11; F[k]=$12; D[k]=$14; BP[k]=$15; MF[k]=$10; R[k]=$19; key[$1" "$2" "$3" "$4" "$5]=1 }
END {
 for (x in key) { split(x,a," "); to=a[4]; tc=a[5]; if (to==tc) continue; c=a[1]" "a[2]" "a[3]; dg=c" "to" "to; dt=tc-to
  d1=S[x" inc"]-S[dg" inc"]; if (d1<0) d1=-d1; if (d1>h1) h1=d1
  dr=A[x" inc"]-A[dg" inc"]-dt*D[dg" inc"]; bar=0.05*(dt<0?-dt:dt)*BP[dg" inc"]; if (bar<1e-5) bar=1e-5; q=(dr<0?-dr:dr)/bar; if (q>h2) h2=q
  if (dr>dmax) dmax=dr; if (dr<dmin) dmin=dr
  rf=(A[dg" phi"]+dt*D[dg" phi"])/(A[dg" inc"]+dt*D[dg" inc"])
  if (MF[dg" phi"]=="true" && tc>to) { e=R[x" phi"]/rf-1; if (e<0) e=-e; if (e>h4a) h4a=e }
  if (tc==0.2 && MF[dg" phi"]=="false" && R[dg" phi"]>1) { r=R[x" phi"]; if (!n5a++ || r<lo5) lo5=r; if (r>hi5) hi5=r }
  if (tc<to && (R[x" phi"]-1)*(R[dg" phi"]-1)<=0) { r=R[x" phi"]; if (!n5c++ || r<lo5c) lo5c=r; if (r>hi5c) hi5c=r; s=S[x" phi"]-S[dg" phi"]; if (!m5c++ || s<los) los=s; if (s>his) his=s }
  # thrust sign vs bleed sign
  fr=F[x" phi"]/F[x" inc"]; if ((fr-1)*(R[x" phi"]-1)<0) th++ ; tot++
 }
 printf "H1 max |dsm_inc| %.2e\nH2 max |drift|/bar %.3f (drift %.2e..%.2e)\nH4a max |R/Rf-1| %.4f\nH5a R %.4f..%.4f (n=%d)\nH5c misses R %.4f..%.4f (n=%d), phi sm rise %.4f..%.4f\nthrust ratio on other side of 1 than bleed: %d of %d off-diagonal\n", h1, h2, dmin, dmax, h4a, lo5, hi5, n5a, lo5c, hi5c, n5c, los, his, th, tot
}
