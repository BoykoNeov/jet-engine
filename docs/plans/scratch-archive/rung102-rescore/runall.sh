D="W:/temp/claude/rung102-rescore"
for law in pin ellipse two one; do
  cmd //v:on //c "start /belownormal /b /wait W:\temp\claude\rung102-rescore\target\release\score_c.exe $law & exit !errorlevel!" > "$D/score_c_$law.txt" 2> "$D/score_c_$law.err"; echo "score_c $law exit $?"
  cmd //v:on //c "start /belownormal /b /wait W:\temp\claude\rung102-rescore\target\release\tt4hold.exe $law & exit !errorlevel!" > "$D/tt4hold_$law.txt" 2> "$D/tt4hold_$law.err"; echo "tt4hold $law exit $?"
done
