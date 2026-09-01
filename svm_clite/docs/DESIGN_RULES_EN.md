# C-Lite design rules

C-Lite is not intended to become a small C compiler. It is a **structured, architecture-independent assembly language** that hides the syntax of the nine SVM assembly targets.

The primary constraints are language simplicity and compiler simplicity. A feature belongs in C-Lite only when it lowers directly to a small number of existing CLIR operations. Features requiring SSA, a register allocator, optimizer passes, a complex type system, or target-specific algorithms are out of scope.

There is no optimizer. Source operations are lowered mechanically to CLIR and then to target assembly.

`bool` remains deliberately simple: `true`/`false` in source, `.bool` in CLIR, one byte containing 0 or 1 in memory, and no bit packing. A backend may later map a transient boolean directly to a native flag/carry/predicate only when that is a direct local mapping, not a separate optimization pass.

## Backend simplicity and code quality

A direct backend means more than merely removing a canonical pseudo-ISA. A short-lived CLIR temporary should remain in the target's natural machine state when that is simple: data stack, accumulator, belt slot, or a small fixed expression-register set. Unnecessary temporary RAM is not the desired default.

This is still not an optimizer: there is no later pattern matching or global analysis, only direct target-specific representation and instruction selection.
