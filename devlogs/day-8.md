# Day 8
I made a big discovery today, I FOUND A SUITABLE `no_std` WASM RUNTIME!!!!
This gave me a lot of motivation to finish, and then I started work on
heap allocation

## Heap Allocation
Today was mostly focussed on heap allocation, at the end of the day
I understood how to implement a bump allocator, and added support
for linked list allocation with `linked_list_allocator`. I also
Added `good_memory_allocator` for increased performance, and 
made the allocator configurable at compile time through the usage
of Cargo feature flags.
