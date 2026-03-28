/-
  Lamp Sync — Verified properties

  These theorems correspond to P1-P7 in docs/sync-architecture.md.
  We prove them about the pure sync function, not about I/O.
-/

import LampSync.Types
import LampSync.Merge
import LampSync.Sync

namespace LampSync

/-!
## P1: Quiescence

If there are no remote changes, the sync engine produces no actions.
(The "no local modifications" part of P1 is about the push phase, which
the caller handles before invoking `sync`. The pull phase — modeled here —
is quiescent when the remote change list is empty.)
-/

theorem quiescence (locals : List Task) :
    sync locals [] = [] := by
  simp [sync, List.flatMap]

/-!
## P5: Merge determinism

`mergeContent` is a pure function — same inputs, same output.
This is trivially true in Lean (all functions are pure), but we
state it explicitly to match the spec.
-/

theorem merge_deterministic (l₁ l₂ r₁ r₂ : TaskContent) :
    l₁ = l₂ → r₁ = r₂ → mergeContent l₁ r₁ = mergeContent l₂ r₂ := by
  intro hl hr
  rw [hl, hr]

/-!
## P6: Logbook/clock preservation (non-conflict merge path)

After merge, the logbook contains all entries from both local and remote.
We prove the superset property for logbook (clock is analogous).
-/

theorem logbook_superset_remote (local remote : TaskContent) :
    ∀ entry, entry ∈ remote.logbook →
      entry ∈ (mergeContent local remote).logbook ∨
      -- entry may have been deduped but is semantically present
      ∃ e ∈ (mergeContent local remote).logbook, e == entry = true := by
  intro entry hmem
  simp [mergeContent, setUnion]
  sorry  -- requires lemma about dedup preserving membership

/-!
## P4: Hash stability (partial)

If we merge and store the hash, then on the next cycle with the same remote
content and no local edits, the stored hash matches the recomputed hash.

This is the core "hash after merge" invariant.
-/

/-- A task is "clean" if its stored hash matches its content hash. -/
def isClean (t : Task) : Prop :=
  t.sync.syncHash = some (contentHash t.content)

/-- A clean, non-dirty task will not trigger a conflict when the same remote arrives. -/
theorem hash_stability (localTask : Task) (href : Href) (etag : ETag)
    (remoteContent : TaskContent)
    (h_clean : isClean localTask)
    (h_not_dirty : localTask.sync.syncDirty = false)
    (h_href : localTask.sync.syncHref = some href)
    -- The remote content, after merge with local, produces the same content
    -- (i.e., local was the result of a previous merge with this remote)
    (h_stable : mergeContent localTask.content remoteContent = localTask.content) :
    ∀ action ∈ processChange [localTask] (.changed href etag remoteContent),
      ∃ task, action = SyncAction.save task := by
  sorry  -- requires unfolding processChange with the hypotheses

/-!
## P7: Idempotent pull (statement)

Applying the same remote changes twice (with the results of the first
applied as the new local state) produces the same state.
-/

-- This requires modeling the "apply actions to local state" step,
-- which we'll add once the core proofs are complete.

/-!
## P3: No silent data loss (statement)

A locally modified task is never deleted without producing a conflict.
Proof depends on I1 (dirty flag consistency), modeled as a hypothesis.
-/

/-- A task is locally modified if its hash differs from stored hash. -/
def isLocallyModified (t : Task) : Prop :=
  t.sync.syncHash ≠ none ∧ contentHash t.content ≠ t.sync.syncHash.get!

/--
  If a task is locally modified and remote deletes it,
  the sync engine emits a conflict (not a delete).
-/
theorem no_silent_delete (localTask : Task) (href : Href)
    (h_exists : findByHref [localTask] href = some localTask)
    (h_modified : isLocallyModified localTask) :
    ∀ action ∈ processChange [localTask] (.deleted href),
      ∃ l r, action = SyncAction.conflict l r := by
  sorry  -- requires unfolding processChange and using h_modified

end LampSync
