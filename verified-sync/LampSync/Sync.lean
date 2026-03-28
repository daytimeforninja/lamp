/-
  Lamp Sync — Core sync algorithm

  Pure function: given local tasks and remote changes, produce a list of
  actions. The caller (Rust/Kotlin) executes actions against the DB and server.

  This models Phase 2 (Pull) from docs/sync-architecture.md.
  Phase 1 (Push) is handled by the caller before invoking this function.
  Phase 3 (Reconstruct) is handled by the caller after executing actions.
-/

import LampSync.Types
import LampSync.Merge

namespace LampSync

/--
  Abstract content hash function. We axiomatize it as a parameter rather than
  implementing FNV-1a, because the proofs don't depend on the hash algorithm —
  only on the property that identical content produces identical hashes.
-/
axiom contentHash : TaskContent → ContentHash

/-- Hash determinism: same content → same hash. -/
axiom contentHash_deterministic :
  ∀ (a b : TaskContent), a = b → contentHash a = contentHash b

/-- Hash sensitivity: different hash → different content. Contrapositive of determinism. -/
theorem contentHash_contrapositive :
    ∀ (a b : TaskContent), contentHash a ≠ contentHash b → a ≠ b := by
  intro a b hne heq
  exact hne (contentHash_deterministic a b heq)

/-- Find a local task by its sync_href. -/
def findByHref (tasks : List Task) (href : Href) : Option Task :=
  tasks.find? fun t => t.sync.syncHref == some href

/--
  Process a single remote change against the local task list.
  Returns the actions to take.
-/
def processChange (locals : List Task) (change : RemoteChange) : List SyncAction :=
  match change with
  | .changed href etag remoteContent =>
    match findByHref locals href with
    | none =>
      -- New remote task: save with hash
      let newTask : Task := {
        id := { val := 0 }  -- caller assigns real ID
        content := remoteContent
        sync := {
          syncHref := some href
          syncEtag := some etag
          syncHash := some (contentHash remoteContent)
          syncDirty := false
        }
      }
      [.save newTask]
    | some localTask =>
      -- Existing task: merge, check for conflict
      let merged := mergeContent localTask.content remoteContent
      let mergedHash := contentHash merged
      let localHash := contentHash localTask.content

      let mergedTask : Task := {
        id := localTask.id
        content := merged
        sync := {
          syncHref := some href
          syncEtag := some etag
          syncHash := some mergedHash
          syncDirty := false
        }
      }

      if localTask.sync.syncHash == none then
        -- Unknown baseline → accept remote
        [.save mergedTask]
      else if localHash == localTask.sync.syncHash.get! then
        -- BEq comparison: local unchanged → accept remote
        [.save mergedTask]
      else
        -- Both sides changed → conflict
        [.conflict localTask mergedTask]

  | .deleted href =>
    match findByHref locals href with
    | none => []  -- Already gone, no-op
    | some localTask =>
      let localHash := contentHash localTask.content
      if localTask.sync.syncHash != none &&
         localHash != localTask.sync.syncHash.get! then
        -- Locally modified → conflict
        [.conflict localTask localTask]
      else
        -- Unmodified or unknown baseline → accept delete
        [.delete localTask.id]

/-- Process all remote changes, producing the full action list. -/
def sync (locals : List Task) (changes : List RemoteChange) : List SyncAction :=
  changes.flatMap (processChange locals)

end LampSync
