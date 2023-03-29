# Day 15
This day went great I got everything working again! No more double faults!!,
turns out it was due to a general protection fault (which don't cause
double faults anymore), which occurred due to the `StackSegment` not
being set in the `GDT`. I also got mouse support working!
