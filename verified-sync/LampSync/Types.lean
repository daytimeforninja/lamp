/-
  Lamp Sync — Domain types

  These model the task and sync metadata types from docs/sync-architecture.md.
  We use simplified representations suitable for proving properties about the
  sync algorithm, not for runtime execution. Dates and times are opaque —
  their internal structure doesn't matter for sync correctness.
-/

namespace LampSync

/-- Task state in the GTD state machine. -/
inductive TaskState where
  | todo
  | next
  | waiting
  | someday
  | done
  | cancelled
  deriving Repr, BEq, DecidableEq

/-- Priority levels (org-mode convention). -/
inductive Priority where
  | a
  | b
  | c
  deriving Repr, BEq, DecidableEq

/-- Opaque date type. We don't need date arithmetic for sync proofs. -/
structure Date where
  val : Nat
  deriving Repr, BEq, DecidableEq

/-- Opaque datetime type. -/
structure DateTime where
  val : Nat
  deriving Repr, BEq, DecidableEq

/-- A clock entry: (start, end) pair of work sessions. -/
structure ClockEntry where
  start : DateTime
  stop : DateTime
  deriving Repr, BEq, DecidableEq

/-- Opaque recurrence specification. -/
structure Recurrence where
  val : String
  deriving Repr, BEq, DecidableEq

/-- Opaque unique identifier. -/
structure TaskId where
  val : Nat
  deriving Repr, BEq, DecidableEq

/-- Opaque content hash (FNV-1a, 64-bit). -/
structure ContentHash where
  val : UInt64
  deriving Repr, BEq, DecidableEq

/-- CalDAV resource href. -/
structure Href where
  val : String
  deriving Repr, BEq, DecidableEq

/-- CalDAV ETag for optimistic concurrency. -/
structure ETag where
  val : String
  deriving Repr, BEq, DecidableEq

/--
  Task content fields — the fields included in the content hash.
  Merge strategies are documented per field.
-/
structure TaskContent where
  title       : String                    -- LWW (remote)
  state       : TaskState                 -- LWW (remote)
  priority    : Option Priority           -- LWW (remote)
  contexts    : List String               -- LWW (remote)
  scheduled   : Option Date               -- LWW (remote)
  deadline    : Option Date               -- LWW (remote)
  notes       : String                    -- LWW (remote)
  project     : Option String             -- remote ?? local
  waitingFor  : Option String             -- LWW (remote)
  esc         : Option Nat                -- LWW (remote)
  delegated   : Option Date               -- LWW (remote)
  followUp    : Option Date               -- LWW (remote)
  recurrence  : Option Recurrence         -- LWW (remote)
  extraTags   : List String               -- G-Set union
  logbook     : List DateTime             -- G-Set union
  clockEntries: List ClockEntry           -- G-Set union
  dayplanDate : Option Date               -- LWW (remote)
  dayplanBudget : Option Nat              -- LWW (remote)
  completed   : Option DateTime           -- LWW (remote)
  deriving Repr, BEq

/-- Sync metadata — not included in content hash. -/
structure SyncMeta where
  syncHref  : Option Href
  syncEtag  : Option ETag
  syncHash  : Option ContentHash
  syncDirty : Bool
  deriving Repr, BEq

/-- A task is content + metadata + identity. -/
structure Task where
  id      : TaskId
  content : TaskContent
  sync    : SyncMeta
  deriving Repr, BEq

/-- A remote change from the CalDAV server. -/
inductive RemoteChange where
  | changed (href : Href) (etag : ETag) (content : TaskContent)
  | deleted (href : Href)
  deriving Repr

/-- An action the sync engine tells the caller to execute. -/
inductive SyncAction where
  | save (task : Task)
  | delete (id : TaskId)
  | push (task : Task)
  | conflict (local : Task) (remote : Task)
  deriving Repr

end LampSync
