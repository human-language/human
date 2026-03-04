# Philosophy

## Orthogonality

Every feature must compose cleanly with every other feature. No special cases. If adding feature B requires a carve-out in feature A, then either B is wrong or A is. Orthogonality is not a luxury — it is the only thing that scales.

## Minimalism as a Forcing Function

If a feature can't be explained in one sentence, it doesn't belong in v1. This is not aesthetic preference — it is engineering discipline. Every feature you add is a feature you must support, document, test, and defend forever. The cost of addition is permanent. The cost of omission is temporary.

## Parsability

The language must be easy to parse — by machines and humans equally. If a human has to think about whether a line is valid, the syntax has failed. If a parser needs backtracking or lookahead beyond one token, the grammar has failed.

## Unix Tools Philosophy

Tools do one thing and do it well, chaining through simple interfaces: pipes, files, stdin/stdout. Human files must be plain text, composable, and pipeable. If you can't `cat` two files together and get something meaningful, the file model is broken.

## Abstraction Must Earn Its Place

Every layer of indirection has a cost. That cost is paid in complexity, in debugging time, in the cognitive load of every person who reads the code after you. Do not add abstraction because it is elegant. Add it because without it, the system cannot be understood.

## Simplicity First

Always ask: "what is the simplest possible thing that could work?" before entertaining complexity. If the simple thing works, ship it. If it doesn't, you now know exactly why, and you can add complexity with precision instead of speculation.

## Skepticism as Kindness

Premature feature addition is technical debt. Every feature someone requests is a hypothesis — that it will be needed, that it will be used correctly, that it will compose with everything else. Most hypotheses are wrong. Saying "not yet" is not obstruction. It is the only honest response to uncertainty.
