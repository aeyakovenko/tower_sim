---
title: Vote Subcommittee
---

## Problem

Each additional validator adds 1 vote per slot, increasing the
message and compute load on the network.

## Solution Overview

Allow a sampled set of validators to vote on that block as opposed
to all the validators, and achieve probabilistically similar liveness
properties as if all the validators vote.

## Detailed Solution

The following sections provide more details of the design.

### Definitions

* Voting Subcommittee: the set of nodes currently voting on blocks

* primary subcommittee: The half of the voting subcommittee that
is scheduled for its second epoch.

* secondary subcommittee: The half of the voting subcommittee that
is scheduled for its first epoch.

* subcommittee seed: The seed used to generate the random sample of
nodes. `slow_hash(penultimate snapshot hash, voting epoch number)`

* super root: The min of the primary and secondary super-majority
roots.

* SRI - super root increase: When the super root is increased by
any number of super roots between the parent and child bank.

### Subcommittee Rotation

```

a0 a1 A1 A1 a1 a2 A2 A2 a2 a3
B1 B1 b1 b2 B2 B2 b2 b3 B3 B3
```

Voting subcommittee is composed of a **primary** and **secondary**
committees. The rotation occurs after N **SRIs**. The transition
is activated in the child bank that detects the Nth **SRI** on its
creation, and it is active for that child bank.

Network should be stable with N = 1. The epoch boundary depends on
the number of **SRIs**, and it is not a fixed number of slots or a
fixed number of roots. At N=1 the **primary rotation** is likely
to take 1 root, and the **secondary rotation** likely to take many
roots as the **secondary** catches up with the **primary**.

#### Primary Rotation

In this rotation, the current **primary** and **secondary** flip.

#### Secondary rotation

In this rotation, the current **primary** stays constant and
**secondary** rotates to a new randomly sampled subcommittee.

### Heaviest fork

Heaviest fork is always determined by the **primary votes**. When
evaluating the bank state, the bank's activated primary determines
the fork weight. A new primary is activated only after it has been
**secondary** and has fully caught up to the existing **primary**
and rooted some slots with the previous active **primary**. Thus
the bank state will always reflect the recent vote history of the
current **primary**.

### Optimistically Confirmed Safety

BOTH **primary** and **secondary** must have 2/3+ votes on the same
fork.

### Optimistically Confirmed Liveness

Only the primary votes may be used for switching proofs.
