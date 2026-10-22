#!/bin/sh
# HEAD then memo build, same 400 requests (seed 31)
cmd //v:on //c "start /belownormal /b /wait W:\temp\claude\gov-slow\probe\target\release\bytes.exe W:/temp/claude/gov-slow/head.txt 400 31 & exit !errorlevel!"
cmd //v:on //c "start /belownormal /b /wait W:\temp\claude\gov-slow\probe2\target\release\bytes.exe W:/temp/claude/gov-slow/memo.txt 400 31 & exit !errorlevel!"
cmp /w/temp/claude/gov-slow/head.txt /w/temp/claude/gov-slow/memo.txt && echo IDENTICAL
