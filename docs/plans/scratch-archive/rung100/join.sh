#!/bin/sh
# join shipped grid / formula with low-grid by key; print per case: grid_shipped, grid_low, formula (start miss, end miss)
D=/w/temp/claude/rung100-probe; G=/w/temp/claude/rung100-probe-grid
awk -F' [|] ' '
FNR==NR { split($1,k," "); key=k[1]" "k[2]" "k[3]" "k[4]" "k[5]" "k[6]; low[key]=$2; next }
{ split($1,k," "); key=k[1]" "k[2]" "k[3]" "k[4]" "k[5]" "k[6];
  if (k[7]=="grid") g[key]=$2; else f[key]=$2; keys[key]=1 }
END { for (key in keys) printf "%-34s | grid %-22s | low %-22s | formula %s\n", key, g[key], low[key], f[key] }
' $G/gridlow_low.txt $D/gridlow_shipped.txt | sort
