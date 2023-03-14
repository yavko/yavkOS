# Day two
## Pre research
Today I started off with doing research into wasm runtimes,
to see if I could use any with this project, I remembered 
that theseus-os (another rust based OS), had created their
own fork of wasmtime to work under `no_std`, but this probably
wouldn't work under my setup anyways, I think I might try using
google's https://github.com/google/wasefire, which seems to 
relatively small

## Code writing
After the research, I started adding in CPU exceptions based on blog os.
After adding CPU exception handlers, I had to add double fault handlers,
which are like another level of handling exceptions, and if not handled
cause a system reset, AKA reboot.

After adding those I started on adding hardware interrupts, after adding
the timer interrupt, I stopped for the day.
