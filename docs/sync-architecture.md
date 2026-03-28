# Lamp Sync Architecture

This document specifies the sync model used by Lamp's desktop (Rust) and mobile
(Kotlin/Android) clients. It serves as a reference for the formally verified
sync engine planned in `verified-sync/`.

## Overview

Lamp syncs tasks between devices via a CalDAV server. Each client maintains a
local database of tasks. The sync protocol is **server-mediating** (not
peer-to-peer): clients push local changes and pull remote changes through a
shared CalDAV collection, using content hashes for change detection and ETags
for optimistic concurrency.

```
             CalDAV Server
           (source of truth for transport)
              /          \
        push /            \ push
            /    pull      \    pull
    Desktop DB            Mobile DB
    (Rust/GTK)            (Kotlin/Room)
```

## Data Model

### Task

The unit of sync. Each task has content fields and sync metadata:

**Content fields** (included in hash):

| Field          | Type                  | Notes                              |
|----------------|-----------------------|------------------------------------|
| title          | String                |                                    |
| state          | TODO/NEXT/WAITING/SOMEDAY/DONE/CANCELLED | GTD state machine    |
| priority       | Option<A/B/C>         | Org-mode priorities                |
| contexts       | List<String>          | GTD contexts (@home, @office, ...) |
| scheduled      | Option<Date>          |                                    |
| deadline       | Option<Date>          |                                    |
| notes          | String                |                                    |
| project        | Option<String>        |                                    |
| waiting_for    | Option<String>        |                                    |
| esc            | Option<u32>           | Energy/spoon cost (5-100)          |
| delegated      | Option<Date>          |                                    |
| follow_up      | Option<Date>          |                                    |
| recurrence     | Option<Recurrence>    |                                    |
| extra_tags     | List<String>          | Non-context tags                   |
| logbook        | List<DateTime>        | Completion timestamps (habits)     |
| clock_entries  | List<(DateTime,DateTime)> | Work timer sessions            |
| dayplan_date   | Option<Date>          |                                    |
| dayplan_budget | Option<u32>           | Spoon budget (first task only)     |
| completed      | Option<DateTime>      |                                    |

**Sync metadata** (not hashed):

| Field     | Type           | Notes                                 |
|-----------|----------------|---------------------------------------|
| sync_href | Option<String> | CalDAV resource URL                   |
| sync_etag | Option<String> | Server ETag for optimistic concurrency|
| sync_hash | Option<u64>    | Content hash at last sync baseline    |
| sync_uid  | Option<String> | CalDAV UID (preserved for roundtrip)  |
| sync_dirty| bool           | Local modification flag               |

### Content Hash

FNV-1a over content fields in deterministic order. Both platforms must produce
identical hashes for identical inputs.

```
FNV offset basis: 0xcbf29ce484222325
FNV prime:        0x100000001b3

for each byte in input:
    hash ^= byte
    hash *= prime
```

Fields are written in this order:
1. title (raw UTF-8 bytes)
2. state keyword (raw UTF-8 bytes)
3. priority (optional: 0x00 for None, 0x01 + org string bytes for Some)
4. each context (raw UTF-8 bytes, in order)
5. scheduled (optional: 0x00 or 0x01 + "YYYY-MM-DD")
6. deadline (optional: 0x00 or 0x01 + "YYYY-MM-DD")
7. notes (raw UTF-8 bytes)
8. project (optional)
9. waiting_for (optional)
10. esc (optional: 0x00 or 0x01 + decimal string)
11. delegated (optional date)
12. follow_up (optional date)
13. recurrence (optional string)
14. each extra_tag (raw UTF-8 bytes, in order)
15. each logbook entry ("YYYY-MM-DDTHH:MM:SS")
16. each clock entry: start then end ("YYYY-MM-DDTHH:MM:SS")
17. dayplan_date (optional date)
18. dayplan_budget (optional int)
19. completed (optional: 0x00 or 0x01 + "YYYY-MM-DDTHH:MM:SS")

Cross-platform test vectors exist in both codebases to prevent drift.

## Sync Algorithm

A sync cycle has three phases executed in order: **push**, **pull**, **reconstruct**.

### Phase 1: Push

Push runs first so that local edits reach the server before we pull. This
prevents the common case of a local edit being immediately overwritten by a
stale remote version.

#### 1a. Push dirty tasks

For each task where `sync_dirty = true`:

```
if sync_href is None:
    # New task, never synced
    href = /{calendar}/{task.id}.ics
    PUT href with If-None-Match: * (create only)
    if 412 (already exists):
        PUT href unconditionally (convert to update)
else:
    # Previously synced task
    PUT sync_href unconditionally (no If-Match)

on success:
    sync_hash  = hash(task)
    sync_etag  = response ETag
    sync_href  = href
    sync_dirty = false

on failure:
    log error, skip (retry next sync)
```

**Dayplan budget stamping** (before push): The first dirty task with
`dayplan_date == today` gets `dayplan_budget = plan.spoon_budget`. All other
dayplan tasks get `dayplan_budget = null`. This encodes the budget once per plan
to save space.

#### 1b. Push deletes

For each task where `sync_deleted = true`:

```
if sync_href is not None and sync_etag is not None:
    DELETE sync_href with If-Match: sync_etag
    if failure:
        log error, skip (retry next sync, keep local tombstone)
        continue

# Only delete locally after remote confirms (or task was never synced)
delete from local DB
```

**Desktop additionally** retries with unconditional PUT on 412 during push.

### Phase 2: Pull

Pull fetches remote changes and applies them to the local database.

#### 2a. Fetch remote changes

```
if sync_token exists and local DB is not empty:
    try sync-collection(token) -> (changes, new_token)
    on token-expired: fall back to full listing
else:
    full listing: list all VTODOs -> changes
    new_token = fresh token from PROPFIND (or None)
```

#### 2b. Apply changes

For each `Changed(remote_vtodo)`:

```
remote = parse(vtodo.ical_body)
local  = lookup by sync_href

if local is None:
    # New remote task
    save remote with:
        sync_href = vtodo.href
        sync_etag = vtodo.etag
        sync_hash = hash(remote)
        sync_dirty = false

if local is not None:
    # Existing task changed remotely
    merged = merge(local, remote)    # see Merge Algorithm below
    merged.sync_href = vtodo.href
    merged.sync_etag = vtodo.etag
    merged.sync_hash = hash(merged)  # IMPORTANT: hash AFTER merge

    local_hash = hash(local)

    if local.sync_hash is None or local_hash == local.sync_hash:
        # Local unchanged (or unknown baseline) -> accept remote
        save merged with sync_dirty = false
    else:
        # Both sides changed -> conflict
        emit Conflict(local, merged)
```

For each `Deleted(href)`:

```
local = lookup by sync_href

if local is None:
    # Already gone locally, no-op
    pass

if local is not None:
    local_hash = hash(local)

    if local.sync_hash is not None and local_hash != local.sync_hash:
        # Locally modified -> conflict (let user decide)
        emit Conflict(local)
    else:
        # Unmodified or unknown baseline -> accept remote delete
        delete local
```

#### Key invariant: hash after merge

The sync_hash stored on a task must be the hash of the *merged* content, not
the raw remote content. This is because the merge may add local logbook
entries, clock entries, or extra tags. If we hashed before merge, the stored
hash would not match the actual content, causing a false conflict on the next
sync cycle.

### Phase 3: Reconstruct

After pull, derived data structures are rebuilt from the task collection:

1. **Projects**: Scan all tasks for distinct `project` values, create missing
   `Project` records.
2. **Habits**: Tasks with `recurrence`, logbook entries, or "habit" tag are
   upserted into the habit table. Completions are merged (union + dedup).
3. **Day plan**: Tasks with `dayplan_date == today` are collected. Their IDs
   form the confirmed task list. The budget is extracted from the first task
   carrying `dayplan_budget`.

## Merge Algorithm

When a remote change arrives for an existing local task, fields are merged:

```
merge(local, remote) -> merged:
    # Start from remote (server-authoritative for content fields)
    merged = remote

    # Preserve local identity
    merged.id = local.id

    # Project: remote wins if present, otherwise keep local
    merged.project = remote.project ?? local.project

    # Tags: union if remote has any, otherwise keep local
    merged.extra_tags =
        if remote.extra_tags is empty: local.extra_tags
        else: deduplicate(remote.extra_tags + local.extra_tags)

    # Logbook: always union (both devices' completions matter)
    merged.logbook = deduplicate(sort(remote.logbook + local.logbook))

    # Clock: always union (both devices' work sessions matter)
    merged.clock_entries = deduplicate(remote.clock + local.clock)
```

**Desktop additionally** performs a three-way merge when the base (sync_hash
baseline) is available, using `merge_local_fields()`. The merge is
server-authoritative for scalar fields (title, state, priority, etc.) and
union-based for collection fields (logbook, clock, tags).

## Conflict Types

| Type          | Trigger                                         | Resolution               |
|---------------|-------------------------------------------------|--------------------------|
| StateMismatch | Local hash != sync_hash AND remote changed      | User picks local/remote  |
| LocalOnly     | Remote deleted but local was modified            | User picks keep/delete   |
| RemoteOnly    | Remote task exists with no local match (desktop) | Auto-import or user pick |

## Wire Format

Tasks are serialized as iCalendar VTODO components (RFC 5545). Standard fields
map to standard VTODO properties. GTD-specific fields use `X-LAMP-*` extension
properties:

| Property           | Maps to          | Example                          |
|--------------------|------------------|----------------------------------|
| X-LAMP-STATE       | state            | `X-LAMP-STATE:NEXT`              |
| X-LAMP-PROJECT     | project          | `X-LAMP-PROJECT:Q1 Review`       |
| X-LAMP-WAITING-FOR | waiting_for      | `X-LAMP-WAITING-FOR:Alice`       |
| X-LAMP-ESC         | esc              | `X-LAMP-ESC:40`                  |
| X-LAMP-DELEGATED   | delegated        | `X-LAMP-DELEGATED:20260310`      |
| X-LAMP-FOLLOW-UP   | follow_up        | `X-LAMP-FOLLOW-UP:20260318`      |
| X-LAMP-RECURRENCE  | recurrence       | `X-LAMP-RECURRENCE:+1w`          |
| X-LAMP-TAGS        | extra_tags       | `X-LAMP-TAGS:urgent,review`      |
| X-LAMP-LOGBOOK     | logbook          | `X-LAMP-LOGBOOK:20260301T090000` |
| X-LAMP-CLOCK       | clock_entries    | `X-LAMP-CLOCK:...T100000/...T113000` |
| X-LAMP-DAYPLAN     | dayplan_date/budget | `X-LAMP-DAYPLAN:20260315;BUDGET=80` |

## Error Recovery

| Error                  | Trigger              | Recovery                                   |
|------------------------|----------------------|--------------------------------------------|
| Sync token expired     | 403/409/412 + XML    | Full listing, acquire fresh token           |
| Stale ETag (412)       | PUT with old ETag    | Desktop: retry unconditional. Mobile: retry next sync |
| Network failure        | Any request          | Log, skip, retry next sync cycle            |
| Unparseable VTODO      | Malformed iCal       | Skip task, continue with remaining          |
| Remote delete fails    | DELETE returns error  | Keep local tombstone, retry next sync       |

## Platform Differences

| Aspect                 | Desktop (Rust)               | Mobile (Kotlin)              |
|------------------------|------------------------------|------------------------------|
| Push condition         | If-Match with ETag, retry 412 unconditionally | Unconditional (no If-Match) |
| Three-way merge        | Yes (merge.rs)               | No (two-way: remote wins + union) |
| Conflict detection     | Post-sync state comparison   | Inline during pull           |
| Done task handling     | Skip pulling completed tasks | Pull all tasks               |
| Delete on complete     | DELETE from server on DONE   | Keep on server               |

These differences are candidates for unification in the verified sync engine.

## Properties to Prove

The following properties should hold for any correct implementation of this
sync algorithm:

### P1: Quiescence

If no side has local modifications (`sync_dirty = false` for all tasks, no
pending deletes), a sync cycle produces no actions (no PUTs, no DELETEs, no
conflicts, no database writes).

### P2: Convergence

After applying all actions from a sync cycle, a second immediate sync cycle
produces no actions. The system reaches a fixed point in at most two rounds.

### P3: No silent data loss

A task that has been locally modified (`sync_dirty = true` OR `hash(task) !=
sync_hash`) is never deleted or overwritten without either:
- Being successfully pushed to the server first, OR
- Producing a conflict for user resolution

### P4: Hash stability

`hash(merge(local, remote))` stored as `sync_hash` equals `hash(task)` on the
next sync cycle (assuming no local edits). This is the "hash after merge"
invariant that prevents false conflicts.

### P5: Merge determinism

`merge(local, remote)` is a pure function: same inputs always produce the same
output, regardless of platform or invocation order.

### P6: Logbook/clock preservation

For any sync cycle, the logbook and clock entries in the result are a superset
of the union of local and remote entries. No work tracking data is ever lost.

### P7: Idempotent pull

Pulling the same remote state twice (without intervening local edits) produces
the same local state. `apply(apply(state, remote), remote) = apply(state, remote)`.

## Verified Sync Engine Plan

The goal is to extract the sync decision logic into a formally verified Lean 4
module that both platforms call via FFI:

```
CalDAV client (Rust/Kotlin)
    |
    | fetches remote state, provides local state
    v
Lean 4 sync engine (pure function, formally verified)
    inputs:  List<LocalTask>, List<RemoteChange>, SyncMetadata
    outputs: List<SyncAction>
                = Save(task) | Delete(id) | Push(task) | Conflict(local, remote)
    |
    | actions executed by caller
    v
App layer (Rust/Kotlin) -> DB writes, HTTP requests
```

The Lean module proves P1-P7 by construction. The Rust and Kotlin callers are
responsible only for I/O and serialization, which are validated by differential
testing (Cedar pattern): generate random sync scenarios, run through both the
Lean engine and the native implementation, assert identical action lists.
