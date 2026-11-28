# kc's database: what others built (a survey)

The user, 2026-11-28, after C563 left a warm kc-steps sweep slower than a
cold one (TODO §59): "do any of them cover revisiting the design of the
database itself, like its fundamentals?", then "do research on the
internet, especially academia, about the design options here". What
follows is that survey, made by a research agent and kept as it wrote it;
`parallel-db.md` is the design it is weighed against (`kayak-db 5`), and
§7 below is what kayak takes from it.

Three things the survey did not know, from `parallel-db.md`:

- Its first pitfall (records naming each other by a writer's own numbers)
  `kayak-db 5` already avoids: a file names keys by their text, labelled
  in that file only (§3a there).
- FX-26 at the pin has no append, no delete, no exclusive create and no
  lock (§2 there); kc removes files by `run-command`'s `rm`. So a design's
  lock files and `O_EXCL` need a Fixpt extension first, or a design that
  needs none (two compactions at once being harmless, as §3 there says).
- That note's §3, before the heads were built, was log-structured with no
  heads, at the user's question "is a log structured database not an
  option?"; the survey's D is close to it.

Date: 2026-11-28. Scope: designs for a persistent query-result store that about 16 concurrent
compiler processes share. It is built on plain files, rename, fsync and possibly fcntl locks, with
no embedded DB library.

Notation used below:

- **A** lazy roots: open reads only the heads' indexes and fetches groups on demand.
- **B** single owner or daemon.
- **C** append-only per-writer files per group, unioned on read, compacted periodically, no heads.
- **D** a new option, proposed at the end. It is C made lazy, plus a bounded run count per group
  that geometric or tiered compaction maintains. The files are self-validating and
  content-keyed, so no global snapshot is needed.

Confidence note: items marked "(verified)" were checked against primary documentation or source
during this survey. The rest come from well-known papers and docs as I recall them; their
citations are standard, but check the details before relying on them.

---

## 0. The diagnosis in one paragraph

The current store is, in the vocabulary of section 3, a **state-based grow-only-set CRDT that
merges on read**. In the vocabulary of section 2, it is a **size-tiered LSM with no compaction
bound and no per-run filters**: every head is a run, and every open pays read amplification
proportional to the number of runs (about 45 ms × 16). The literature is consistent that merging
on read is cheap for writers and expensive for readers, and that it is only affordable when
readers do one of two things. They either touch only the keys they need (laziness, plus fence
pointers or Bloom filters), or the run count stays logarithmic through compaction. The present
design does neither: it merges eagerly, at open, across all groups. So the fix is not "stop
merging". It is to **merge lazily and per key or group, and to keep runs per group bounded**.

---

## 1. Build systems and compiler caches with concurrent writers

### 1.1 Bazel: disk cache, remote cache, action cache, server

- **Citation:** Bazel docs, "Remote caching" (https://bazel.build/remote/caching) and "Client/server
  implementation" (https://bazel.build/run/client-server). Disk-cache GC design doc (draft, 2024),
  and `src/tools/diskcache/` in the Bazel tree. (verified)
- **Mechanism:** The remote and disk caches have two maps. The **AC** maps action digest to
  ActionResult; the **CAS** maps content digest to blob. Every entry is immutable and keyed by a
  hash, so concurrent writers of the same key write identical bytes, and "last rename wins" is
  harmless. The disk cache is a directory of hash-named files written to a temp file and then
  renamed; several Bazel servers may share it. GC is by mtime/atime age and size (Bazel 7.4
  `--experimental_disk_cache_gc_max_size/_max_age`). The GC design doc says an index is needed
  only to make GC fast and to coordinate it, not to serve lookups. The doc reaches for SQLite
  there and excludes NFS because of its locking. Separately, the **local action cache** and the
  in-memory Skyframe graph belong to one server per output base, which holds an exclusive lock.
  A second client blocks ("Another command is running") or fails fast.
- **Mapping:** Bazel gets concurrency in two layers. The **shared** layer is a lookup-per-key,
  content-addressed file store with no snapshot, no heads and no merging. That is the D/C
  direction, and it shows that an open-time merge is unnecessary when entries are deterministic
  and keyed. The **rich incremental** state (the dep graph) is single-owner (B) and is never
  shared between concurrent processes. Bazel never merges dep graphs from concurrent writers.

### 1.2 Buck2 and DICE

- **Citation:** "Modern DICE" (https://buck2.build/docs/insights_and_knowledge/modern_dice/), slides
  at https://buck2.build/assets/Modern_DICE.pdf. (verified)
- **Mechanism:** DICE (Distributed Incremental Computation Engine) is an in-memory,
  Salsa/Adapton-style engine with versioned keys, parallel compute, work sharing, dependency
  tracking and early cutoff. It lives in the long-running buck2 daemon. Persistence across daemon
  restarts goes through the remote/disk action cache (CAS plus AC), not by serializing the DICE
  graph. A small local SQLite tracks materializer state.
- **Mapping:** This is a strong data point for **B for the graph, a keyed shared cache for the
  outputs**. Meta's choice was to make the daemon the unit of incrementality, and to accept that
  restarting the daemon loses the graph, with the content cache keeping rebuilds cheap.

### 1.3 Shake and "Build Systems à la Carte"

- **Citation:** N. Mitchell, "Shake before building", ICFP 2012. A. Mokhov, N. Mitchell,
  S. Peyton Jones, "Build Systems à la Carte", ICFP 2018; extended as "Build systems à la carte:
  Theory and practice", JFP 30, 2020. Shake source `Development.Shake.Internal.Core.Storage` and
  the shake-build-system mailing list (2020). (verified for locking)
- **Mechanism:** Shake keeps one `.shake.database` file plus an append-only **journal**. Each
  completed rule is appended as a length-prefixed record and flushed. On open, Shake replays the
  journal into the database, and it compacts on clean exit. A torn tail record is detected and
  dropped. A `.shake.lock` file excludes concurrent processes: the maintainer states that "only one
  thing should be mutating it at once". The à la Carte paper splits a build system into a
  **scheduler** and a **rebuilder**, with **verifying traces, constructive traces and deep
  constructive traces**. Constructive traces store (key, dep-hashes, result), so a store of them is
  a deterministic memo table that can be shared across machines ("cloud build systems").
- **Mapping:** The journal-with-crash-safe-tail is exactly the right per-writer primitive for C/D.
  The paper's trace model says that an entry keyed by (query, input fingerprints) is
  self-validating. A reader never needs a consistent snapshot, only the trace for the key it is
  verifying. That argues against needing heads at all. Shake itself chose single-writer exclusion
  over concurrent writers, because its database also stores per-rule ordering and indexes.

### 1.4 Nix store locking

- **Citation:** E. Dolstra, *The Purely Functional Software Deployment Model*, PhD thesis, Utrecht,
  2006. Nix and Lix `libstore/pathlocks.cc`. (verified)
- **Mechanism:** Store paths are immutable and named by hash. Concurrent builders take a
  **per-output-path lock file** (`<path>.lock`, fcntl). The loser of the race waits, then
  re-checks validity rather than rebuilding. Validity metadata lives in a SQLite DB, and the
  nix-daemon serializes multi-user access to it.
- **Mapping:** Per-key locks are only for **deduplicating work**, never for correctness, because
  identical outputs make races benign. Nix uses an owner (the daemon, B) for its mutable metadata
  index, while the immutable content stays plain files. That is a hybrid of B and D.

### 1.5 ccache

- **Citation:** ccache manual (https://ccache.dev/manual/latest.html) and the samba ccache list,
  2018 thread on cleanup and stats contention. (verified for stats and cleanup)
- **Mechanism:** Results live in a 2-level hash-prefix directory tree. Writers create a temp file
  and rename it into place, and there is no global index. Statistics are sharded into one stats
  file per subdirectory, each under its own lock, "to make this more scalable". Even so, the
  developers report lock contention on those few files. Cleanup runs per subdirectory, when that
  subdirectory exceeds 1/16th of the quota, using mtime. Version 4 introduced lock files with a
  keep-alive (staleness detection), so that a killed holder does not wedge others.
- **Mapping:** This is D-like (per-key files, no snapshot). It also warns that **any shared mutable
  aggregate** (counters, indexes) becomes the contention point, so shard it and make it advisory.

### 1.6 sccache

- **Citation:** sccache README and docs/Local.md and Configuration.md (https://github.com/mozilla/sccache).
  (verified)
- **Mechanism:** The client auto-spawns a local server, which exits after `SCCACHE_IDLE_TIMEOUT`
  (default 600 s). The server owns an in-memory LRU index over a disk directory. The docs warn
  that the local storage supports only one server at a time: "multiple concurrent servers will
  race and cause spurious build failures". Mozilla's bugzilla has "Timed out waiting for server
  startup" flakes.
- **Mapping:** This is B in its simplest form, and it shows B's characteristic failure modes:
  startup races, timeouts, and correctness that depends on there being exactly one owner. Its
  on-disk format is not safe for multiple owners, which is the opposite of what D aims for.

### 1.7 rustc incremental compilation

- **Citation:** `compiler/rustc_incremental/src/persist/fs.rs` module docs
  (https://doc.rust-lang.org/nightly/nightly-rustc/rustc_incremental/persist/fs/index.html); rustc-dev-guide
  "Incremental compilation in detail"; PR #147821 (2025, session GC vs flock). (verified)
- **Mechanism:** Each compiler session creates a fresh `s-<timestamp>-<random>-working` directory,
  holds an fcntl lock on its lock file, and **hard-links** (copy-on-write) the files of the newest
  finalized session into it. On success it renames the directory to its finalized name. On open, a
  session picks **the newest finalized directory** and **never merges** concurrent sessions'
  results. GC runs at every start and deletes finalized directories that are not newest. It also
  deletes `-working` directories whose lock can be taken, which means their owner died. A reader
  copying a directory takes a shared lock, and GC takes an exclusive one. PR #147821 fixed GC
  deleting the session's own fresh directory, because fcntl locks are per-process and so do not
  exclude the same process. The docs warn that locking on NFS is unreliable.
- **Mapping:** rustc treats concurrent sessions as **last writer wins on the whole snapshot**,
  accepting lost work. It does this because its dep-graph uses session-relative `SerializedDepNodeIndex`
  numbering, so two graphs cannot be unioned. This is the key pitfall for any union design: **records
  must not reference each other by writer-relative indexes**. If they reference each other by
  stable hashes (DefPathHash, Fingerprint), union is sound. The fcntl-per-process subtlety applies
  directly to a store implemented in-process.

### 1.8 Salsa persistence and rust-analyzer

- **Citation:** rust-analyzer issue #4712 "Persistent caches";
  "Durable Incrementality" (rust-analyzer blog, 2023-07-24); salsa `persistence` feature and
  `tests/persistence.rs` (salsa 0.26). (verified)
- **Mechanism:** Salsa is an in-memory red-green engine with revision counters and "durability"
  levels. Durability is an optimization that skips revalidating rarely changing inputs; it is
  unrelated to disk. Recent Salsa versions have an experimental `persistence` feature that
  serializes the whole database, opt-in per query, as a single snapshot written by one process.
  rust-analyzer still has no on-disk cache. Its maintainers framed the open questions as mmap vs.
  parse-into-memory, and "best-effort cache; if unusable, recompute".
- **Mapping:** No precedent here for multi-writer persistence. The idea of durability levels does
  matter, though. Partition the store by **volatility**: stdlib/dependency queries vs. workspace
  queries. The rarely changing partition can be compacted into one big immutable run (cheap to
  open, lazily read), and churn goes elsewhere. This maps well onto D's per-group tiering.

### 1.9 Pluto, PIE and other incremental-computation work

- **Citations:**
  - S. Erdweg, M. Lichter, M. Weiel, "A Sound and Optimal Incremental Build System with Dynamic
    Dependencies", OOPSLA 2015 (Pluto).
  - G. Konat, S. Erdweg, E. Visser, "Scalable Incremental Building with Dynamic Task Dependencies",
    ASE 2018.
  - G. Konat, M. Steindorfer, S. Erdweg, E. Visser, "PIE: A DSL, API and Runtime for Interactive
    Software Development Pipelines", <Programming> 2(3), 2018.
  - Konat, Sol, Erdweg, Visser, "Precise, Efficient, and Expressive Incremental Build Scripts with
    PIE", 2019.
  - M. Hammer et al., "Adapton: Composable, Demand-Driven Incremental Computation", PLDI 2014.
  - U. Acar, *Self-Adjusting Computation*, PhD thesis CMU 2005.
  - "Incremental Computation: What Is the Essence?", arXiv 2312.07946.
  - H. Esfahani et al., "CloudBuild: Microsoft's Distributed and Caching Build Service",
    ICSE-SEIP 2016.
  (Pluto/PIE verified)
- **Mechanism:** Pluto checks consistency **top-down and must traverse the whole previous
  dependency graph** on every build. PIE's ASE'18 bottom-up algorithm starts from changed files
  and touches only affected tasks, so cost scales with the change, not the graph. PIE persists its
  store through a pluggable key-value store, the default being LMDB, so it inherits LMDB's
  single-writer, many-reader model. Adapton and Acar are about in-memory demand-driven graphs.
  CloudBuild shares a content-keyed cache across thousands of machines with no graph sharing.
- **Mapping:** This is the most important lesson about **open cost**. Pluto's scaling problem is the
  same shape as reading every head at open: **work proportional to the whole stored state, not to
  the change**. PIE's fix is to read lazily, on demand from the change set. It argues for A and D.
  Specifically, a warm build that re-executes 4% of queries should touch about 4% of the stored
  state plus a validation frontier, and should never deserialize it all. Profile whether
  red/green validation itself, not just open, walks the whole graph.

### 1.10 Go build cache (GOCACHE)

- **Citation:** `cmd/go/internal/cache` package docs and `cache.go` (https://pkg.go.dev/cmd/go/internal/cache).
  R. Cox, "go build cache" design (Go 1.10 release notes, 2017). (verified for the mechanism)
- **Mechanism:** It is two content-addressed maps in plain files: `actionID → (outputID, size,
  time)` as small `-a` files, and `outputID → bytes` as `-d` files. These live under 256 hash-prefix
  directories. Writes are write-then-rename, or careful in-place write with size checks. Reads
  validate the entry and treat any corruption as a miss. There is **no lock, no index, no
  manifest, no snapshot**, and many concurrent `go` processes share the cache safely. Usage is
  recorded by bumping mtime, at most once per hour per file to limit writes. Trim runs about
  once a day (recorded in `trim.txt`) and deletes entries unused for 5 days. It is explicitly
  designed so that one process's trim never deletes something another process recently used.
- **Mapping:** This is the purest existence proof for **"no heads at all; the filesystem
  directory is the index"**. It is C without per-writer files: one file per key, with idempotent
  overwrite. Open cost is zero, and lookup is one `open`. The cost is many small files. That is
  fine for Go, with its coarse package-level granularity, but possibly too fine for per-query
  records, hence the grouping in D.

### 1.11 Other caches worth knowing

- **Gradle build cache and daemon** (docs.gradle.org "Build Cache", "Gradle Daemon"): the local cache
  is a directory of hash-named entries with a cross-process lock file (Gradle's `FileLockManager`,
  which uses a lock-contention ping over UDP to ask the holder to release). Daemons are discussed
  in section 4.
- **Zig's cache** (`std.Build.Cache`): per-input-hash manifest files under `h/`, each guarded by
  flock. Writers take an exclusive lock on the manifest being produced, and readers a shared lock.
  This is per-key locking with no global state, the same shape as D.
- **Turborepo and Nx local caches:** hash-named directories, rename-into-place, no index.

---

## 2. Storage engines on plain files

### 2.1 LSM trees: leveling vs. tiering, per-writer runs vs. a shared log

- **Citations:**
  - P. O'Neil, E. Cheng, D. Gawlick, E. O'Neil, "The Log-Structured Merge-Tree (LSM-Tree)", Acta
    Informatica 33(4), 1996.
  - N. Dayan, M. Athanassoulis, S. Idreos, "Monkey: Optimal Navigable Key-Value Store", SIGMOD
    2017.
  - N. Dayan, S. Idreos, "Dostoevsky: Better Space-Time Trade-Offs for LSM-Tree Based Key-Value
    Stores via Adaptive Removal of Superfluous Merging", SIGMOD 2018.
  - C. Luo, M. Carey, "LSM-based Storage Techniques: A Survey", VLDB Journal 2020.
  - LevelDB and RocksDB docs (MANIFEST/CURRENT, `LOCK`, log format).
  - L. Lu et al., "WiscKey: Separating Keys from Values in SSD-Conscious Storage", FAST 2016.
- **Mechanism:** Writes go to immutable sorted runs, and reads must consult every run that might
  hold the key. With **tiering**, up to T runs per level accumulate before merging: writes are
  cheap, and reads cost O(T·L) run probes. With **leveling** there is one run per level, so reads
  are cheap and writes amplified. Monkey shows that per-run **Bloom filters** with tuned bits
  per level make most probes of non-matching runs nearly free. **Fence pointers** (a small index
  per run) make a probe one seek, not a scan. Dostoevsky's *lazy leveling* tiers the small levels
  and levels the largest. LevelDB and RocksDB track the live set of runs in an append-only
  **MANIFEST** log, with an atomic `CURRENT` file renamed to point at it. A `LOCK` file
  (fcntl) admits exactly one writer process, and RocksDB offers read-only and "secondary"
  instances for other processes.
- **On per-writer runs vs. a shared log:** a shared log needs a single appender, or atomic
  multi-process append, which O_APPEND gives only for small writes and only on local filesystems.
  That forces a lock or an owner. Per-writer runs need no coordination but multiply the run count,
  which is exactly the present 16-head read amplification. LSM theory says this is fine
  **if** (1) each run is probed only for keys being looked up, via fence pointers and Bloom filters,
  and (2) compaction keeps the run count O(log N) or bounded by T.
- **Mapping:** The current design is tiering with T = number of concurrent writers and no filters,
  and it reads every run eagerly. That is the worst corner of the design space. **A** turns eager
  reads into probes. **C** is per-writer runs per group, with compaction bounding T. **D** adds
  per-group fence indexes and an explicit run-count bound. LevelDB's single-writer LOCK is the
  standard choice, and it argues that a mutable shared log needs B.

### 2.2 LMDB-style copy-on-write B-trees

- **Citation:** H. Chu, "MDB: A Memory-Mapped Database and Backend for OpenLDAP", LDAPCon 2011, and
  "LMDB" talks (symas.com). Related: O. Rodeh, "B-trees, Shadowing, and Clones", ACM TOS 2008.
- **Mechanism:** It is an append-only, copy-on-write B+tree in one file, with two meta pages. A
  commit writes new pages, fsyncs them, then writes and fsyncs the alternate meta page with a
  higher txn id. On open, the meta page with the higher valid id wins, so a torn commit just
  reverts. There is **one writer at a time** (a process-shared mutex in a lock file) and lock-free
  MVCC readers, which register in a reader table so that freed pages are not reused under them.
  Stale reader slots from dead processes are cleared by PID checks.
- **Mapping:** This argues for a **single writer at a time but many processes**: writers take an
  fcntl lock only for the commit, not the whole session. It is a middle ground between B and D.
  With 16 writers saving in short bursts at the end of a compile, a commit lock would serialize
  saves but not compiles. The costs are random-page-write-heavy saves and the need for mmap and a
  free-page allocator, which is heavy for a small language. The two-meta-page trick is the useful
  takeaway: it gives crash-safe root switching with no rename at all.

### 2.3 Bitcask

- **Citation:** J. Sheehy, D. Smith, "Bitcask: A Log-Structured Hash Table for Fast Key/Value Data",
  Basho Technologies, 2010.
- **Mechanism:** Values are appended to the active data file, and older files are immutable. An
  in-memory **keydir** maps key to (file, offset, size). Startup rebuilds the keydir by scanning
  **hint files**, the compact per-file key indexes written at merge time, instead of the data.
  Merge compacts immutable files and drops dead entries. CRCs on each record make the tail
  crash-safe: truncate at the first bad record. There is one writer process per directory, under
  a lock file.
- **Mapping:** The hint file is exactly option **A's** "read only the heads' indexes". Bitcask's
  lesson is that open cost then scales with **number of keys**, not bytes, and is still O(total)
  at every open. That is fine for a long-lived server, and poor for 16 short-lived processes each
  opening. Laziness per group (D) beats a full hint scan.

### 2.4 Git: packfiles, multi-pack-index, commit-graph chains, reftable

- **Citations:** Git technical docs, in Documentation/technical and at git-scm.com/docs:
  - `pack-format`;
  - `multi-pack-index` ("incremental MIDX chains");
  - `commit-graph` ("split commit-graph chains", `--size-multiple`, `--split=merge-all`);
  - `reftable`, by S. Pearce (JGit, 2017), adopted as git's ref backend in 2.45, 2024.
  (verified)
- **Mechanism:**
  - Objects are content-addressed, and each immutable packfile has a sorted `.idx` (with a fanout
    table) for O(log n) lookup without reading the pack.
  - The **MIDX** is one index over many packs, so lookups don't probe every pack's index. The
    *incremental* MIDX keeps a **chain** of layers listed in a chain file, so writes add a small
    tip instead of rewriting.
  - **Split commit-graph chains** merge the new tip into its predecessor whenever
    `size_multiple × N > M`, with a default multiple of 2. That keeps the chain length
    logarithmic: **geometric compaction**.
  - **Reftable** replaced loose refs and `packed-refs`. In packed-refs, a single lookup scans the
    file linearly: about 410 ms cold on 866k refs. Reftable keeps sorted, block-indexed tables, so
    a lookup is about 34 µs.
  - Reftable updates add a new small table to a **stack** listed in `tables.list`. The writer
    takes `tables.list.lock` (O_EXCL create) and renames it over. Auto-compaction keeps the stack
    geometric. Readers that see a stale list retry.
  - Git avoids reading all refs via sorted, indexed tables with prefix seeks. Older git avoided it
    via loose ref files: one file per ref, so lookup is one `open`, just like the Go cache.
- **Mapping:** Git and reftable are a nearly exact blueprint for **D**:
  - immutable, sorted, self-indexed runs;
  - a tiny list file naming the live runs, replaced atomically under a short O_EXCL lock;
  - geometric compaction to keep the list length O(log writes);
  - readers that binary-search only the runs and blocks they need.

  The present "heads" correspond to reftable tables. Reftable shows that bounding the count
  (compaction) and indexing each one (lazy probe) is enough. A shared list costs one short lock
  per save, and git accepts that. If even that is unacceptable, the per-group list can be replaced
  with a directory listing, as in C.

### 2.5 Append-only logs with crash-safe tails, and crash-consistency pitfalls

- **Citations:**
  - LevelDB `doc/log_format.md`: 32 KiB blocks, records with CRC32C and fragment types.
  - SQLite "Write-Ahead Logging" and "Atomic Commit" docs: WAL frames are salted and checksummed,
    and recovery stops at the first bad frame.
  - C. Mohan et al., "ARIES", ACM TODS 1992.
  - T. S. Pillai et al., "All File Systems Are Not Created Equal: On the Complexity of Crafting
    Crash-Consistent Applications", OSDI 2014.
  - T. S. Pillai et al., "Application Crash Consistency and Performance with CCFS", FAST 2017.
  - D. Luu, "Files are hard" (danluu.com/file-consistency), 2015.
- **Mechanism:** Frame each record as length + checksum (+ optional salt or epoch). On recovery,
  scan forward and truncate logically at the first invalid record. Pillai et al. found
  crash-consistency bugs in many mature systems (LevelDB, git, HDFS, SQLite in some modes and
  others). The common causes were missing fsync of the file before rename, missing fsync of the
  **parent directory** after rename or create, and assuming appends are atomic or ordered.
- **Mapping:** All of A to D rely on "write temp, fsync file, rename, fsync directory". The
  brief's crash model is processes killed mid-write, not power loss. For that model, rename alone
  is atomic and fsync is not required for consistency: page-cache contents survive a process
  kill. fsync is needed only for durability against power loss or a kernel crash. For a *cache*,
  that is optional if every record is checksummed and corruption reads as a miss. This is a large
  perf lever: **skip fsync entirely, checksum everything, treat damage as a miss.** That is what
  ccache and the Go cache effectively do.

---

## 3. Merge on read vs. merge on write; CRDT and union semantics

- **Citations:**
  - M. Shapiro, N. Preguiça, C. Baquero, M. Zawirski, "Conflict-free Replicated Data Types",
    SSS 2011, and the companion INRIA RR-7506 "A comprehensive study of CRDTs".
  - P. S. Almeida, A. Shoker, C. Baquero, "Delta State Replicated Data Types", JPDC 2018.
  - H. Sanjuán, S. Pöyhtäri, P. Teixeira, I. Psaras, "Merkle-CRDTs: Merkle-DAGs meet CRDTs",
    arXiv 2004.00107, 2020.
  - P. Bailis, A. Ghodsi, "Eventual Consistency Today", CACM 2013.
  - J. Hellerstein, P. Alvaro, "Keeping CALM: When Distributed Consistency Is Easy", CACM 2020.
  - Athanassoulis et al., "Designing Access Methods: The RUM Conjecture", EDBT 2016.
- **Mechanism:** If the state is a join-semilattice (here a grow-only map whose values for equal
  keys are equal, since deterministic queries give a G-Set of (key, value) pairs), then merge is
  union. Union is commutative, associative and idempotent, so **any** order or duplication of
  merges converges, and no coordination is needed (CALM: monotone programs need no coordination).
  Delta-state CRDTs ship and store only the **deltas** since the last sync rather than full states,
  which is exactly C's per-writer appends. Merkle-CRDTs, IPFS's design for go-ds-crdt, store
  deltas as nodes of a Merkle DAG whose heads are tracked like git commits. That is precisely the
  current fork-and-merge design, and the same papers note that **reading requires walking from all
  heads**, so the head count must be kept small, by merging heads into a new root. The RUM
  conjecture frames the general trade-off: you can optimize at most two of read, update and
  memory overhead. Merge-on-read buys update cost with read cost.
- **Mapping and arguments:**
  1. Correctness is not the problem. Union is a valid merge, provided that (a) values are truly
     deterministic, so the same key always gives the same bytes, and that is worth asserting in
     debug builds; (b) records reference each other by stable content keys, not writer-local
     indexes (the rustc pitfall, 1.7); and (c) GC deletion is not part of the lattice. Deletion
     breaks monotonicity. In CRDT terms it needs tombstones or a causal-stability rule, and the
     "older than 60 s" rule is an ad hoc causal-stability guess. In D, GC deletes only *files*
     whose contents are subsumed by a compacted file, which is monotone at the logical level.
  2. A deterministic cache does not need snapshot consistency across groups. A reader that sees
     group X from writer 1 and group Y from writer 2 is fine, because each entry self-validates
     against its input fingerprints. This removes the main reason for heads.
  3. **Merge-on-write** (compaction) and **merge-on-read** are the same union. The design choice
     is only *when* it happens and *who* pays. The literature's answer for read-heavy stores with
     bursty writers: merge on read lazily, and merge on write in the background, bounded and
     opportunistic.

---

## 4. Daemon and single-owner designs: lifecycle and crash handling

- **Bazel server** (bazel.build/run/client-server; verified): one server JVM per output base. The
  client takes a shared lock on the install base and an exclusive lock on the output base. A
  version or startup-option mismatch makes the client restart the server. If the server dies, the
  client starts a new one and the in-memory graph is lost, while the on-disk action cache and the
  disk/remote caches survive. Concurrent commands serialize, and the documented workaround for
  parallel CI scripts is a separate output base for each, which loses sharing.
- **Gradle daemon** (docs.gradle.org/current/userguide/gradle_daemon.html): a client finds a
  *compatible* idle daemon in a registry file (`registry.bin`, lock-protected) or spawns one. A
  busy daemon is not shared: a second build gets a second daemon. Daemons expire after 3 h idle or
  under memory pressure, and a health monitor stops daemons whose GC thrashes. There is a long
  history of stale or incompatible daemon accumulation.
- **Watchman** (facebook.github.io/watchman): a per-user daemon on a well-known socket in a state
  directory, spawned on first use. Its state is reconstructible: after a restart it re-crawls and
  answers "since clock X" queries with a "fresh instance" flag, so clients must treat a restart as
  "everything may have changed". That is a clean model: **the daemon holds only cache, never truth.**
- **rust-analyzer:** the editor owns one process per workspace. There is no persistence and no
  sharing between instances, and the maintainers declined to build a persistent daemon (1.8).
- **sccache server** (1.6): spawned on demand, exits after 10 min idle, has startup-timeout races,
  and only one may own a cache directory.
- **Buck2 daemon** (1.2): one per project root and isolation dir, holding DICE in memory. Users
  may run isolated daemons, which, as with Bazel, do not share.
- **Common lessons:**
  1. A daemon makes concurrent *clients* easy only if it serves them concurrently. Bazel and Gradle
     serialize or duplicate per build, so they don't really solve 16-way parallelism.
  2. The lifecycle tax is version and option compatibility, a discovery and lock protocol, idle
     timeouts, health monitoring, and crash recovery with clients retrying.
  3. Every successful design keeps the daemon's state **reconstructible from an on-disk format
     that is itself safe**. So B does not remove the need for a good on-disk store; it adds a
     layer over one.
  4. Throughput: 16 processes doing fine-grained query lookups over IPC pay a round trip per
     lookup unless the protocol batches. Bazel avoids that by putting the *compiler* (Skyframe) in
     the daemon. For kc, the equivalent would be running compilations as threads in one server,
     which is a much larger change than a storage daemon.
- **Mapping:** B is the right long-term shape if kc becomes a resident compile server, with
  incrementality in memory, as in Bazel, Buck2 and rust-analyzer. As a mere *storage* daemon for
  16 independent compiler processes, B adds IPC latency per lookup and lifecycle complexity. It
  still needs a crash-safe on-disk format, which leads back to D.

---

## 5. Other fundamentally different approaches

1. **One file per key, content-addressed (Go cache, ccache, loose git objects and refs, Bazel disk
   cache).** There is no open cost at all, and the directory is the index. The costs are inode
   count, per-file syscall overhead and directory fan-out. This works when keys are coarse;
   group-level files are the compromise.
2. **Numbered commit log with optimistic concurrency (Delta Lake, Apache Iceberg).**
   - Citation: M. Armbrust et al., "Delta Lake: High-Performance ACID Table Storage over Cloud
     Object Stores", PVLDB 13(12), 2020 (verified); Iceberg table spec.
   - A commit is the atomic creation of `_delta_log/<N+1>.json`, via put-if-absent (locally,
     `link()` or `O_CREAT|O_EXCL`). A loser re-reads and retries, which for a union store always
     succeeds trivially.
   - Every 10 commits a **checkpoint** holds the full state. Readers read the newest checkpoint
     plus at most about 10 small deltas, so open cost is bounded no matter how many writers.
   - Applied here: heads become a linear sequence of numbered deltas plus periodic checkpoints.
     There is no DAG, and the merge is "checkpoint ∪ tail". That is a strong, simple alternative
     to A, called D' below.
3. **Hard-link snapshot sessions (rustc).** Each process works privately, the newest finalized
   snapshot wins, and others' work is discarded. That is simple and crash-safe, but it loses
   concurrent writers' work. With 16 parallel processes on *different* crates or modules, that
   matters, unless the store is split per compilation unit.
4. **Partition by owner (per-unit stores).** If each of the 16 processes mostly compiles a
   different unit, give each unit its own store, so each has a single writer and no merging. A
   small, shared, read-mostly store holds cross-unit queries (stdlib, dependency metadata),
   compacted into one immutable run. rustc and Cargo work this way: one incremental dir per crate
   and profile. It is worth checking how much cross-process sharing actually occurs.
5. **Shared-memory hash tables over files (e.g. mmap'd open-addressing tables, as in "Hash tables
   for process-shared caches").** Not recommended for a small language without atomics, and
   fragile under kill -9.
6. **Durability tiers (Salsa) and size-tiered compaction keyed by volatility**: put rarely
   changing entries in large compacted runs, and churn in small ones (1.8, 2.1 Dostoevsky).

---

## 6. Assessment and recommendation

### Recommended: D. Lazy, per-group runs with bounded count and opportunistic compaction; no global heads

**On disk:**

- `groups/<bucket-prefix>/<group-hash>/` holds the group's live **runs**:
  `<gen>-<time>-<pid>.run`. Each run is immutable, sorted by key, and framed with per-record
  checksums. It has a trailing footer holding a fence index, an optional Bloom filter and a whole-file checksum.
  This is option C's per-writer file, made self-indexing.
- Optionally there is one small `groups/.../<group-hash>.list`, reftable-style, naming the live
  runs. It is rewritten with `O_EXCL` lock-file plus rename only by compaction. Without it, the
  directory listing *is* the list, as in the Go cache and C.

**Writes:** a saving process writes, for each group it touched, one new run containing only its
**new** entries (a delta, as in delta-CRDTs). It writes to a temp name and renames. No head file,
no parent pointers, no global lock.

**Reads:** open does nothing global. The first lookup in a group lists that group's runs, opens
footers and caches them for the process's lifetime. A warm build that touches 4% of queries
touches roughly that fraction of groups. This is the PIE/A lesson.

**Compaction:** when a reader or writer sees more than T runs in a group (say T=4), it tries a
**non-blocking** per-group lock (`O_EXCL` lock file with pid+time, or fcntl `F_SETLK`). If it
gets the lock, it merges the runs into one run, by union with dedupe, renames it in and deletes
the inputs. If it doesn't, it just reads all runs. Use git's `size_multiple=2` rule to keep the
number of runs logarithmic in total writes. Readers that hit ENOENT on a run listed a moment
earlier re-list and retry, as reftable readers do.

**GC:** a compacted run subsumes its inputs, so deleting the inputs is monotone and safe. Reachability GC is no longer needed. Evict whole groups or entries by age, using an mtime bump
at most hourly, as in the Go cache, during a daily trim that never removes anything used in the
last N hours.

**Crash safety:** a killed writer leaves a temp file (GC it by age) or a complete renamed run. A
torn run fails its checksum and is ignored, which counts as a miss. Skip fsync by default: the
crash model is process kill, not power loss, and the data is a cache. Offer a durable mode that
does file fsync, then rename, then directory fsync.

Why D over the alternatives:

- **vs. A:** A still costs O(heads) index reads per open and keeps the DAG, ancestor GC and the
  60 s heuristic. D's laziness is per group and needs no global structure. If you want a
  stepping stone, A plus a Delta-Lake-style checkpoint (D') keeps today's code shape: heads
  become numbered deltas, and every k-th save writes a checkpoint of the head indexes.
- **vs. B:** B adds per-lookup IPC and a daemon lifecycle (sections 1.6 and 4), and it still needs
  a safe on-disk format. Defer B until kc becomes a resident compile server, at which point it
  should hold the *graph* in memory, as in Bazel and Buck2. D is then its backing store, as
  Bazel's disk cache backs Skyframe.
- **vs. C:** D *is* C plus three things the literature insists on: per-run indexes or filters
  (Monkey, reftable), a bound on runs per group via geometric compaction (commit-graph chains,
  Dostoevsky), and lazy group loading (PIE). C without the bound reproduces today's problem one
  group at a time.

### Pitfalls the literature warns about

1. **Writer-relative identifiers in records** (rustc's `SerializedDepNodeIndex`) make union
   unsound. Every cross-record reference must be a stable content key.
2. **Non-determinism leaks**, such as interned-symbol order, hash-map iteration order, absolute
   paths, timestamps or pids in values. Under union, two "equal" keys with different bytes
   silently diverge. Add a debug check that compares bytes on duplicate keys during compaction.
3. **Deletion is non-monotone.** Time-based GC racing a slow reader is the classic bug: rustc PR
   #147821, the Go cache's "don't trim what another process used recently", ccache keep-alive
   locks. Delete only subsumed files, or files idle for longer than any plausible process lifetime.
4. **fcntl lock semantics:** locks are per process and per (inode, pid). Closing *any* fd on the
   file drops all of the process's locks on it, and they do not exclude threads of the same
   process. They are unreliable on NFS (rustc, Bazel GC doc). Prefer `O_EXCL` lock files carrying
   pid+time, with stale-lock breaking, for short critical sections, or `flock`/OFD locks where available.
5. **Crash-consistency details** (Pillai et al., OSDI 2014): rename without a preceding fsync can
   expose a zero-length file after a power loss, and a directory entry is not durable without a
   directory fsync. If you skip fsync, checksums plus treating corruption as a miss are mandatory.
6. **Small-file explosion:** one file per key fails at fine granularity, so group by hash bucket
   and size groups so that a typical run is at least about 4–64 KiB.
7. **Shared mutable aggregates** (global stats, a global index) become hot locks (ccache). Keep
   them sharded and advisory.
8. **Daemon tax** (Gradle, sccache, Bazel): startup races, stale daemons, incompatible options and
   memory growth. Don't take this on for storage alone.
9. **Whole-graph validation** (Pluto): even with a lazy store, a top-down red/green walk that
   visits every stored node will cost O(graph). Bottom-up or demand-driven validation keeps the
   store's laziness meaningful.

### Open questions a profile should answer first

1. **Where do the ~45 ms per head go?** Split the time into directory listing, `open`/`stat`
   counts, bytes read, decoding or allocation in kc's runtime, and hashing. If it is CPU decoding,
   laziness (A or D) wins big. If it is syscall count, fewer and larger files matter more.
2. **How many opens per build?** Multiply per-open cost by the number of compiler process
   invocations, not just by 16. A per-open cost that looks small multiplies across hundreds of
   invocations.
3. **What fraction of groups does a warm build touch?** If it is near 100% because validation
   walks everything, fix validation first, and the store design matters less.
4. **How much is actually shared across concurrent processes?** If most queries are private to
   one compilation unit, per-unit stores (5.4) remove most merging for free.
5. **What is the run-count distribution per group** after a typical parallel build, and how
   duplicated are entries across heads? That sets T and tells you whether compaction is cheap.
6. **Is the 97 s vs. 65 s gap all in open?** Also measure save cost (writing heads and files,
   ancestor GC) and the every-8th-save reachability GC. The fix may be partly in the write path.
7. **Are values byte-deterministic today?** Run a duplicate-key byte comparison over a merged
   database before relying on union semantics without heads.

---

## Source list (primary)

- Bazel remote caching: https://bazel.build/remote/caching ; client/server: https://bazel.build/run/client-server ; disk cache GC tool: https://bazel.googlesource.com/bazel/+/HEAD/src/tools/diskcache/
- Buck2 Modern DICE: https://buck2.build/docs/insights_and_knowledge/modern_dice/
- Mokhov, Mitchell, Peyton Jones, "Build Systems à la Carte", ICFP 2018 / JFP 2020
- Shake storage source: https://hackage.haskell.org/package/shake ; lock discussion: https://groups.google.com/g/shake-build-system/c/aeuAbVBs1uU
- Nix pathlocks: https://git.lix.systems (libstore/pathlocks.cc); Dolstra PhD thesis 2006
- ccache manual: https://ccache.dev/manual/latest.html ; list thread: https://lists.samba.org/archive/ccache/2018q1/001535.html
- sccache docs: https://github.com/mozilla/sccache (docs/Local.md, docs/Configuration.md)
- rustc incremental fs: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_incremental/persist/fs/index.html ; PR https://github.com/rust-lang/rust/pull/147821
- rust-analyzer persistent caches: https://github.com/rust-lang/rust-analyzer/issues/4712 ; Durable incrementality: https://rust-analyzer.github.io/blog/2023/07/24/durable-incrementality.html
- Erdweg et al., Pluto, OOPSLA 2015; Konat et al., ASE 2018: https://repository.tudelft.nl/record/uuid:e167152d-540a-4afc-a6b4-636478c2eebe ; PIE: https://arxiv.org/abs/1803.10197
- Go build cache: https://pkg.go.dev/cmd/go/internal/cache
- O'Neil et al., LSM-tree, Acta Informatica 1996; Dayan et al., Monkey SIGMOD 2017; Dayan & Idreos, Dostoevsky SIGMOD 2018; Luo & Carey, VLDB J 2020
- Chu, LMDB, LDAPCon 2011; Rodeh, ACM TOS 2008
- Sheehy & Smith, Bitcask, Basho 2010
- Git technical docs: https://git-scm.com/docs/reftable ; https://git-scm.com/docs/multi-pack-index ; https://git-scm.com/docs/git-commit-graph ; https://www.kernel.org/pub/software/scm/git/docs/technical/commit-graph.html
- Pillai et al., OSDI 2014; Luu, "Files are hard"
- Shapiro et al., CRDTs, SSS 2011; Almeida et al., delta-state CRDTs, JPDC 2018; Sanjuán et al., Merkle-CRDTs, arXiv 2004.00107; Hellerstein & Alvaro, CALM, CACM 2020; Athanassoulis et al., RUM, EDBT 2016
- Armbrust et al., Delta Lake, PVLDB 13(12) 2020: https://vldb.org/pvldb/vol13/p3411-armbrust.pdf
- Gradle daemon: https://docs.gradle.org/current/userguide/gradle_daemon.html ; Watchman: https://facebook.github.io/watchman/

---

## 7. What kayak took (C565)

**The profile first** (2026-11-28, an idle machine, `perf record -F 199
--call-graph fp` over a warm kc-steps sweep): kc 99% of the samples, and
83% of kc's under `lines-scanned!`, an open putting each head's line of
each group together (`where-inserted`, `where-groups`: a file's name made
a list of characters to be compared; the heap's zeroing and collection
under them). Saving and collecting 9%; the compiling itself under 10%.
Of the survey's open questions: the cost was decoding, not system calls
(§6's 1); four opens a program, about 1,600 a sweep (2); a warm build
reads its own program's group and the runtime's, a few of 889 (3); and
the gap was all in the open (6). Values were already byte-deterministic
and named by content (7), and records by key, not number (pitfall 1).

**The path: D, as far as FX-26 lets it** (`kayak-db 6`):

- A head names no shard: its generation, revision, `last` and parents
  alone, a line. Heads stay for the revision shortcut and its merge
  (`models/db.qnt`), small.
- A group's files are found by the listing (`groups-listed!`), named
  `gMARKHASH.RUN`, MARK whose (a hash of the kc's identity): another kc's
  files are known by name, unread (the Go cache's, ccache's way: the
  directory is the index). A shard is read when a key of it is first
  asked for, as before (lazy per group, PIE's lesson).
- A shard's file names in its first line the files of its group it was
  made of (reftable's stack, Delta Lake's checkpoint, in one): a reader
  takes the newest, skips what it replaces, and puts together any other
  (two runs' at once, C559's union). A group's files so stay few without
  a compaction of their own: every write of a group is one.
- What a save makes old it removes: the files it replaced, and their
  values' files its own does not name. Values at once; shards' files and
  another kc's after a minute (a slower reader may be reading them; one
  left, the group's next reader finds it old again). No head is read
  whole to know what is kept, and C563's one-minute window on a group's
  union is gone.
- Not taken: Bloom filters and fence indexes (a group is one file, read
  whole when asked for, and kc asks for few); a lock (FX-26 has none, and
  two writes of one group are both sound); fsync (`write-file-unflushed`
  already for shards and values, the head flushed).

**Measured** (kc-steps, cold then warm twice, the same machine idle):
before, 69 s, 95 s, 95 s; after, 52 s, 19 s, 20 s, 0 wrong, the
database level at about 3,650 shard files (each program's group written
a few times a sweep, the replaced kept a minute). A warm sweep's profile
after: the open 5%, the save 4.5%; most is `kc run` assembling again the
`asm` values the database does not keep (`asm_image`, 32%), and the
heap's zeroing (`refill`, 41% with what is under it).
