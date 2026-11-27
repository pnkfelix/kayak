# kc's database, safe for processes at once

The user, 2026-11-26, after C551 gave kc-steps a database per worker:
"Regarding kc-steps in parallel: I recommend making the database safe for
parallel access", and "Yeah let's look into parallel-safe database now".
This note is the design, before code.

## 1. What breaks today

`lib/db.kfx` (C472, `kayak-db 3`): a run opens the head, reads shards as
their keys are asked for, works in memory, and at its end writes every
shard a node of which changed (to the other of its two slots) and then the
head. Two runs of kc on one database at once break it three ways:

1. **Lost updates.** Each writes the head whole, as it saw it at its start
   plus its own changes: the last writer's head wins, and the shards the
   other wrote are named by no head. Worse, both write a shard to the same
   "other slot", so a head can name a slot whose file the other run wrote.
2. **Numbers.** A node's number is its shard's number times `shard-span`
   plus its place in the shard, given in order (`db-id`); a shard's number
   is its place in the head, given in order (`db-shard-of-group!`); what a
   node read is kept as numbers. Two runs adding keys to one shard, or
   shards to the head, give the same numbers to different keys.
3. **Revisions.** A node is current if nothing it read changed after the
   revision it was verified in (`db-reads-durability`), or at once if no
   input of its durability changed since (`last`). Two runs from one
   revision count on from it each: after both save, one run's revisions
   say nothing of the other's changes.

## 2. What FX-26 gives to build on

At the pin (3569ce51, `crates/fixpt-fx26/src/standard.rs`): `read-file`,
`list-dir` (names sorted by their bytes), `file-info`, `write-file`
(replaced whole and atomically: written beside and renamed over),
`write-file-unflushed` (C475). No append, no delete, no exclusive create,
no lock, no compare-and-swap. So no run can know it alone writes a file;
and nothing can remove one. Only a Fixpt extension could give either
(memory: FX-26 extensions land in Fixpt first).

## 3. The design: log-structured

The user, after this note's first design (one head per save, naming its
parents, the heads no other names merged at open): "is a log structured
database not an option?" It is, and simpler: `list-dir` finds the
segments, so nothing needs to name them.

**Nodes verified by fingerprints, not revisions (step 1, C556).** Each
node keeps, with each key it read, that key's fingerprint as it read it;
it is current if each, made current first, has that fingerprint still
(rustc's red-green, by fingerprint). A node so is a fact that holds
whatever else is in the database: so any two runs' nodes may be put
together, either kept where both have one key. `models/db.qnt` shows it:
its `merge` puts another run's nodes in, their revisions that run's own;
`fixed` (by fingerprints) holds, `byRevisions` does not. Revisions stay as
a shortcut only: a node verified in this run's revision is not walked
again, and `last` lets a node whose durability no input changed at be
current at once; a merge moves `last` at every durability, so the
shortcut never spans two runs' changes.

**Segments, never overwritten.** A save writes one segment, `s.RUN` (RUN
unique: the process and the clock), holding what that run changed: for
each group (a shard, now) its nodes, by key text, what each read by key
and fingerprint, and their values (or value files named by key hash and
fingerprint). Nothing is appended to and no file is replaced, so a reader
never sees a file half another's.

**Opening.** `list-dir`, then the segments newest first; a group's nodes
are those of the newest segment that has the group, the others' of that
group put in under them where a key is not there (either is sound, by
fingerprint). Numbers are this run's alone, given as keys are met; the
files have none. `last`, at once, is moved at every durability where
the segments of the frontier disagree.

**Compacting.** A save that finds more than some segments (eight, say)
writes, beside its own, one that has all of them, and names the ones it
has (`(replaces s.A s.B ...)`); a reader skips what a segment it read
replaces. Two runs compacting at once write two that replace the same:
harmless, both sound.

**Collecting needs Fixpt.** With no delete, the replaced segments stay:
the directory grows by a segment a save. A Fixpt `delete-file` (small,
and of no lock's troubles) would let a compaction remove what it replaced
after a minute (a slower reader finds a file gone, reads the group as
empty, and its queries run again: never wrong). Until Fixpt has it, the
scripts that make databases (kc-steps, the check) remove old segments
from Python, as they remove whole databases now.

## 3a. Step 2 as built (`kayak-db 5`)

Reading the format against two runs at once found what §3 left open:

- **A node is named in the files by its shard's place in the head times
  `shard-span` plus its place in the shard, and what it read by those
  numbers.** Two runs from one head give the same numbers to different
  keys, so files of two heads cannot be put together while they say
  numbers. `kayak-db 5` says none: a shard's file has its own keys,
  labelled by their place in it, and the keys of other groups its nodes
  read, each by its text once, labelled too (`(reads #J=KEY ...)`); a
  node's reads are labels. A label of another group's key is resolved to
  this run's number only when it is walked (verification), so that a
  shard read does not read every shard its nodes name (C472's measure: a
  run beside the expander's 43 MB shard).
- **A value's own file is named by its key's hash and its fingerprint**,
  `vKEYHASH.FP`, not its node's number: two runs that write one are
  writing the same text.
- **A shard's file is `gHASH.RUN`**, RUN this save's (the clock and the
  process), never written again; **a head `h.GEN.RUN`**, GEN one past its
  parents'. Opening lists the directory and reads the heads of the
  greatest GEN: one, most often; several when runs from one head saved at
  once, put together group by group, each group's newest file (by RUN: a
  file a run wrote is newer than the one it had), and `last` moved at
  every durability. A head of a lesser GEN that is no one's parent (a slow
  run's, others having saved twice since) is left: its work lost, never
  wrong.
- **One group's two versions are not merged node by node** (§3 said a
  union): the newer file is taken whole. Sound either way; only work is
  lost, and kc-steps' workers write the runtime's shared groups at once
  rarely (each program is its own group). A union is for later if
  measures want it.
- **Collecting needs no Fixpt extension**: kc runs commands already
  (`run-command`, its `mkdir -p`), so after a save it removes, by one
  `rm`, each head of a GEN two or more below its own, and each shard and
  value file no head kept names, older than a minute (`file-info`): a
  slower reader of an older head finds a file gone, reads its group as
  empty and runs its queries again.

## 4. What it costs

- Each node's record grows by a fingerprint for each key it read (step 1).
- Opening lists the directory and reads the segments' indexes; after a
  compaction, one.
- Verification walks a node's reads where revisions would have said
  "current" at once, only after a merge moved `last`.
- Until `delete-file`, the directory grows between the scripts' cleanings.

## 5. What it gives

kc-steps' workers on one database, warm from the last check when kc did
not change (as `target/kc-steps.db` was before C551); `kc serve`, `kc
build` and an editor's checks at once on a project's database; and the
first step to kc's queries running in parallel in one process (K8): a
node checked by fingerprint is checked the same whoever made it. And
step 1 alone verifies better: a read whose value went A to B to A no
longer sends its reader to run again (`db-verify`: 276 bodies run, not
318).

## 6. Steps

1. Fingerprints with reads, verification by them (`kayak-db 4`, C556): the
   model first, then `lib/db.kfx`; one head and slots still.
2. Files by run, nodes named in them by key; heads by generation, opened
   by `list-dir` and put together; collecting by kc's own `rm` (C559, §3a;
   no Fixpt `delete-file` needed).
3. kc-steps' workers on one database, kept from check to check.

## 7. The alternatives

- **A head per save, naming its parents** (this note's first design):
  what a log needs no head for, since `list-dir` finds the segments.
- **A Fixpt exclusive create** (`create-file-new`) as a lock: one head,
  no merging; but a run that dies holding it leaves it held, and every
  save waits on every other.
