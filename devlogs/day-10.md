# Day ten

## Planning
### Fully finish multitasking
Today as I said I'm planning on adding support
for adding tasks, even after to task executor
has been executed. But I'm not sure if I'll be able to.

### Add basic WASM execution
I think it's time, I finally start adding support
for WASM execution, the whole purpose of this kernel.

## Actual work
I found a comment under a `blog-os` post describing a `Spawner` for spawning
tasks, instead of storing them in the executor, I think ill try implementing that.
