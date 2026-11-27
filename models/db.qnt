// A model of kc's query database (lib/db.kfx; docs/research/models.md).
//
// The engine's algorithm, abstractly: inputs with a value and a
// durability; queries whose bodies read inputs and other queries, what
// they read depending on what they read; each node remembering what it
// read in order, the revision it was verified in, the one its value
// changed in, and the least durability of its reads. Asking a node: current
// if verified this revision; else current at once if no input of its least
// durability changed since (the durability shortcut); else each read made
// current in order, and if each has the fingerprint it had when the node
// read it (kayak-db 4; before, if none changed after the node was
// verified), the node is current; else its body runs again, and if its value is the same,
// its revision of change is kept (early cutoff).
//
// The bodies are fixed, three levels deep, as recursion is not Quint's:
//   m0, m1, m2  read x_i; give x_i mod 2 (so that edits often leave them
//               as they were)
//   s           reads m0; then m1 and m2 if m0 is 0, else m2 alone
//   u           reads s; then x0 if s > 0; gives s + x0, else 0
//
// The property: every value the database gives is the value a run from
// scratch gives (`answerIsSpec`), and every node the durability shortcut
// would take as current is (`shortcutSound`).
//
// FIXED chooses the engine as fixed or as it was: true, an edit counts at
// the higher of the input's old and new durability (lib/db.kfx since C399,
// the bug tests/kfx/db-verify found), and a node verified by its reads takes
// their least durability as it is now (C400, the bug this model found);
// false, neither.
module dbModel {
  const FIXED: bool
  // BYFP chooses verification: true, by each read's fingerprint as read
  // (kayak-db 4, docs/research/parallel-db.md); false, by revisions.
  const BYFP: bool
  // MERGES: whether another run's nodes are put into the database
  // (`merge`), as a save of a run at once with this one would.
  const MERGES: bool
  // Edits stop at this revision, so that the states are finite for an
  // exhaustive check (TLC); asking goes on.
  const MAXREV: int

  type Input = { value: int, dur: int, changed: int }
  // What a node read: each key and the fingerprint it had when read (a
  // value is its own fingerprint here).
  type Node = { verified: int, changed: int, value: int, deps: List[(str, int)], dur: int }
  type St = { rev: int, inputs: str -> Input, nodes: str -> Node, last: int -> int, runs: int }

  pure val INPUTS = Set("x0", "x1", "x2")
  pure val QUERIES = Set("m0", "m1", "m2", "s", "u")
  pure val DURS = 0.to(2)
  pure val TOP = 2

  // ---------------------------------------------------------------- the spec

  pure def x(st: St, k: str): int = st.inputs.get(k).value
  pure def specM(st: St, i: str): int = x(st, i) % 2
  pure def specS(st: St): int =
    if (specM(st, "x0") == 0) specM(st, "x1") + specM(st, "x2") else specM(st, "x2")
  pure def specU(st: St): int = if (specS(st) > 0) specS(st) + x(st, "x0") else 0
  pure def spec(st: St, k: str): int =
    if (k == "m0") specM(st, "x0")
    else if (k == "m1") specM(st, "x1")
    else if (k == "m2") specM(st, "x2")
    else if (k == "s") specS(st)
    else if (k == "u") specU(st)
    else x(st, k)

  // ---------------------------------------------------------- the engine

  pure def minI(a: int, b: int): int = if (a < b) a else b
  pure def maxI(a: int, b: int): int = if (a > b) a else b

  pure def changedOf(st: St, k: str): int =
    if (INPUTS.contains(k)) st.inputs.get(k).changed else st.nodes.get(k).changed
  pure def durOf(st: St, k: str): int =
    if (INPUTS.contains(k)) st.inputs.get(k).dur else st.nodes.get(k).dur

  // A read, made current, unchanged since the node was verified: by its
  // fingerprint as read and as it is now (`BYFP`, kayak-db 4); or, as
  // before, by its revision of change and the node's of verification.
  pure def unchanged(st: St, kf: (str, int), now: int, since: int): bool =
    if (BYFP) kf._2 == now else changedOf(st, kf._1) <= since

  pure def marked(st: St, k: str): St =
    { ...st, nodes: st.nodes.set(k, { ...st.nodes.get(k), verified: st.rev }) }

  // A node verified by its reads, each made current: its least durability
  // theirs now, which an input's lowered durability may have lowered
  // (`FIXED`; found by this model, C400). Unfixed, as it was.
  pure def reverified(st: St, k: str, dur: int): St =
    { ...st, nodes: st.nodes.set(k, { ...st.nodes.get(k), verified: st.rev,
                                     dur: if (FIXED) dur else st.nodes.get(k).dur }) }

  // A body's reads so far: the state, what it read in order, the least
  // durability, and the last value read.
  type Reading = { st: St, reads: List[(str, int)], dur: int, v: int }

  pure def stored(r: Reading, k: str, v: int): St = {
    val st = r.st
    val old = st.nodes
    val changed =
      if (old.keys().contains(k) and old.get(k).value == v) old.get(k).changed else st.rev
    { ...st,
      nodes: old.put(k, { verified: st.rev, changed: changed, value: v, deps: r.reads, dur: r.dur }),
      runs: st.runs + 1 }
  }

  // Level 0: inputs, always current.
  pure def current0(st: St, k: str): (St, int) = (st, st.inputs.get(k).value)
  pure def read0(r: Reading, k: str): Reading = {
    val got = current0(r.st, k)
    { st: got._1, reads: r.reads.append((k, got._2)), dur: minI(r.dur, durOf(got._1, k)), v: got._2 }
  }
  pure def depsOk0(st: St, deps: List[(str, int)], since: int): (St, bool, int) =
    deps.foldl((st, true, TOP), (acc, kf) =>
      if (not(acc._2)) acc
      else { val got = current0(acc._1, kf._1); (got._1, unchanged(got._1, kf, got._2, since), minI(acc._3, durOf(got._1, kf._1))) })

  // Level 1: m0, m1, m2.
  pure def body1(st: St, k: str): (St, int) = {
    val i = if (k == "m0") "x0" else if (k == "m1") "x1" else "x2"
    val r = read0({ st: st, reads: [], dur: TOP, v: 0 }, i)
    val v = r.v % 2
    (stored(r, k, v), v)
  }
  pure def asked1(st: St, k: str): (St, int) =
    if (not(st.nodes.keys().contains(k))) body1(st, k)
    else {
      val n = st.nodes.get(k)
      if (n.verified == st.rev) (st, n.value)
      else if (st.last.get(n.dur) <= n.verified) (marked(st, k), n.value)
      else {
        val ok = depsOk0(st, n.deps, n.verified)
        if (ok._2) (reverified(ok._1, k, ok._3), n.value) else body1(ok._1, k)
      }
    }
  pure def current1(st: St, k: str): (St, int) =
    if (INPUTS.contains(k)) current0(st, k) else asked1(st, k)
  pure def read1(r: Reading, k: str): Reading = {
    val got = current1(r.st, k)
    { st: got._1, reads: r.reads.append((k, got._2)), dur: minI(r.dur, durOf(got._1, k)), v: got._2 }
  }
  pure def depsOk1(st: St, deps: List[(str, int)], since: int): (St, bool, int) =
    deps.foldl((st, true, TOP), (acc, kf) =>
      if (not(acc._2)) acc
      else { val got = current1(acc._1, kf._1); (got._1, unchanged(got._1, kf, got._2, since), minI(acc._3, durOf(got._1, kf._1))) })

  // Level 2: s, its reads depending on what it read.
  pure def body2(st: St, k: str): (St, int) = {
    val r0 = read1({ st: st, reads: [], dur: TOP, v: 0 }, "m0")
    val m0 = r0.v
    if (m0 == 0) {
      val r1 = read1(r0, "m1")
      val r2 = read1(r1, "m2")
      val v = r1.v + r2.v
      (stored(r2, k, v), v)
    } else {
      val r2 = read1(r0, "m2")
      (stored(r2, k, r2.v), r2.v)
    }
  }
  pure def asked2(st: St, k: str): (St, int) =
    if (not(st.nodes.keys().contains(k))) body2(st, k)
    else {
      val n = st.nodes.get(k)
      if (n.verified == st.rev) (st, n.value)
      else if (st.last.get(n.dur) <= n.verified) (marked(st, k), n.value)
      else {
        val ok = depsOk1(st, n.deps, n.verified)
        if (ok._2) (reverified(ok._1, k, ok._3), n.value) else body2(ok._1, k)
      }
    }
  pure def current2(st: St, k: str): (St, int) =
    if (k == "s") asked2(st, k) else current1(st, k)
  pure def read2(r: Reading, k: str): Reading = {
    val got = current2(r.st, k)
    { st: got._1, reads: r.reads.append((k, got._2)), dur: minI(r.dur, durOf(got._1, k)), v: got._2 }
  }
  pure def depsOk2(st: St, deps: List[(str, int)], since: int): (St, bool, int) =
    deps.foldl((st, true, TOP), (acc, kf) =>
      if (not(acc._2)) acc
      else { val got = current2(acc._1, kf._1); (got._1, unchanged(got._1, kf, got._2, since), minI(acc._3, durOf(got._1, kf._1))) })

  // Level 3: u, which reads an input only sometimes.
  pure def body3(st: St, k: str): (St, int) = {
    val r0 = read2({ st: st, reads: [], dur: TOP, v: 0 }, "s")
    if (r0.v > 0) {
      val r1 = read2(r0, "x0")
      val v = r0.v + r1.v
      (stored(r1, k, v), v)
    } else (stored(r0, k, 0), 0)
  }
  pure def asked3(st: St, k: str): (St, int) =
    if (not(st.nodes.keys().contains(k))) body3(st, k)
    else {
      val n = st.nodes.get(k)
      if (n.verified == st.rev) (st, n.value)
      else if (st.last.get(n.dur) <= n.verified) (marked(st, k), n.value)
      else {
        val ok = depsOk2(st, n.deps, n.verified)
        if (ok._2) (reverified(ok._1, k, ok._3), n.value) else body3(ok._1, k)
      }
    }
  pure def current3(st: St, k: str): (St, int) =
    if (k == "u") asked3(st, k) else current2(st, k)

  // -------------------------------------------------------------- the system

  var st: St
  var answer: { key: str, got: int, want: int }

  action init = all {
    nondet d0 = DURS.oneOf()
    nondet d1 = DURS.oneOf()
    nondet d2 = DURS.oneOf()
    st' = {
      rev: 1,
      inputs: Map("x0" -> { value: 0, dur: d0, changed: 1 },
                  "x1" -> { value: 0, dur: d1, changed: 1 },
                  "x2" -> { value: 0, dur: d2, changed: 1 }),
      nodes: Map(),
      last: DURS.mapBy(i => 1),
      runs: 0
    },
    answer' = { key: "", got: 0, want: 0 },
  }

  // An input set to a value at a durability: a new revision if either is new.
  action edit = {
    nondet k = INPUTS.oneOf()
    nondet v = 0.to(2).oneOf()
    nondet d = DURS.oneOf()
    val old = st.inputs.get(k)
    all {
      st.rev < MAXREV,
      not(old.value == v and old.dur == d),
      val r = st.rev + 1
      val counted = if (FIXED) maxI(old.dur, d) else d
      st' = { ...st,
              rev: r,
              inputs: st.inputs.set(k, { value: v, dur: d, changed: r }),
              last: st.last.keys().mapBy(i => if (i <= counted) r else st.last.get(i)) },
      answer' = answer,
    }
  }

  // A query asked.
  action ask = {
    nondet k = QUERIES.oneOf()
    val got = current3(st, k)
    all {
      st' = got._1,
      answer' = { key: k, got: got._2, want: spec(st, k) },
    }
  }

  // Another run's nodes put in (docs/research/parallel-db.md): a run from
  // this database at once with this one, its inputs as it set them, its
  // nodes made as its asks made them, and its revisions its own count from
  // a revision this one had. Of each key, its node or ours. Our inputs
  // stay, as each run sets its own. The merge moves `last` at every
  // durability, so the shortcut takes no node across it.
  action merge = all {
    MERGES,
    st.rev < MAXREV,
    nondet a0 = 0.to(2).oneOf()
    nondet a1 = 0.to(2).oneOf()
    nondet a2 = 0.to(2).oneOf()
    nondet asked = QUERIES.oneOf()
    nondet taken = QUERIES.powerset().oneOf()
    nondet v = 1.to(MAXREV).oneOf()
    nondet c = 1.to(MAXREV).oneOf()
    val theirInputs = Map("x0" -> { ...st.inputs.get("x0"), value: a0 },
                          "x1" -> { ...st.inputs.get("x1"), value: a1 },
                          "x2" -> { ...st.inputs.get("x2"), value: a2 })
    val theirs = current3({ ...st, inputs: theirInputs, nodes: Map() }, asked)._1.nodes
    val took = taken.intersect(theirs.keys())
    val r = st.rev + 1
    all {
      v <= st.rev,
      c <= v,
      st' = { ...st,
              rev: r,
              nodes: st.nodes.keys().union(took).mapBy(k =>
                if (took.contains(k)) { ...theirs.get(k), verified: v, changed: c } else st.nodes.get(k)),
              last: st.last.keys().mapBy(i => r) },
      answer' = answer,
    }
  }

  action step = any { edit, ask, merge }

  // ----------------------------------------------------------- properties

  // What the database gives is what a run from scratch gives.
  val answerIsSpec = answer.got == answer.want

  // A node the durability shortcut would take as current has the value a
  // run from scratch gives.
  val shortcutSound =
    st.nodes.keys().forall(k => {
      val n = st.nodes.get(k)
      (st.last.get(n.dur) <= n.verified) implies (n.value == spec(st, k))
    })

  // A node verified this revision has the value a run from scratch gives.
  val verifiedSound =
    st.nodes.keys().forall(k => {
      val n = st.nodes.get(k)
      (n.verified == st.rev) implies (n.value == spec(st, k))
    })

  val sound = answerIsSpec and shortcutSound and verifiedSound
}

// The engine as lib/db.kfx has it (kayak-db 4): verified by fingerprints,
// another run's nodes put in.
module fixed {
  import dbModel(FIXED = true, BYFP = true, MERGES = true, MAXREV = 6).*
}

// The engine before C399's and C400's fixes, which the model must find
// wanting.
module unfixed {
  import dbModel(FIXED = false, BYFP = true, MERGES = false, MAXREV = 6).*
}

// Verified by revisions (kayak-db 3) with another run's nodes put in, which
// the model must find wanting: one run's revisions say nothing of the
// other's changes.
module byRevisions {
  import dbModel(FIXED = true, BYFP = false, MERGES = true, MAXREV = 6).*
}

// Verified by revisions, one run alone (kayak-db 3 as it was): sound.
module oneRun {
  import dbModel(FIXED = true, BYFP = false, MERGES = false, MAXREV = 6).*
}
