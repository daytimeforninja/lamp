/-
  Lamp Sync — Merge algorithm

  Two-way merge: remote wins for scalar fields, G-Set union for collections.
  This is the canonical merge from docs/sync-architecture.md.
-/

import LampSync.Types

namespace LampSync

/-- Deduplicate a list, preserving first occurrence order. -/
def dedup [BEq α] : List α → List α
  | [] => []
  | x :: xs => if xs.any (· == x) then dedup xs else x :: dedup xs

/-- Union two lists with deduplication (G-Set merge). -/
def setUnion [BEq α] (a b : List α) : List α :=
  dedup (a ++ b)

/--
  Merge local and remote task content.

  - Scalar fields: remote wins (LWW)
  - project: remote wins if present, otherwise keep local
  - Set-valued fields (extraTags, logbook, clockEntries): G-Set union
-/
def mergeContent (local remote : TaskContent) : TaskContent :=
  { remote with
    project      := remote.project <|> local.project
    extraTags    := if remote.extraTags.isEmpty then local.extraTags
                    else setUnion remote.extraTags local.extraTags
    logbook      := setUnion remote.logbook local.logbook
    clockEntries := setUnion remote.clockEntries local.clockEntries
  }

end LampSync
