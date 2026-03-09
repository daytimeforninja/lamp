package com.lamp.mobile.core.data.sync

import com.lamp.mobile.core.data.credential.CredentialStore
import com.lamp.mobile.core.data.repository.*
import com.lamp.mobile.core.model.Habit
import com.lamp.mobile.core.model.Project
import com.lamp.mobile.core.model.SyncConflict
import com.lamp.mobile.core.model.SyncMetadata
import com.lamp.mobile.core.model.Task
import com.lamp.mobile.core.network.caldav.CalDavClient
import com.lamp.mobile.core.network.caldav.PutCondition
import com.lamp.mobile.core.network.caldav.SyncChange
import com.lamp.mobile.core.network.caldav.SyncTokenExpiredException
import com.lamp.mobile.core.network.carddav.CardDavClient
import com.lamp.mobile.core.network.ical.VtodoConverter
import com.lamp.mobile.core.network.ical.VeventConverter
import com.lamp.mobile.core.network.webdav.WebDavClient
import android.util.Log
import javax.inject.Inject
import javax.inject.Singleton

data class SyncResult(
    val tasksUploaded: Int = 0,
    val tasksDownloaded: Int = 0,
    val tasksDeleted: Int = 0,
    val eventsDownloaded: Int = 0,
    val contactsSynced: Int = 0,
    val notesSynced: Int = 0,
    val conflicts: List<SyncConflict> = emptyList(),
    val errors: List<String> = emptyList(),
)

@Singleton
class SyncEngine @Inject constructor(
    private val credentialStore: CredentialStore,
    private val taskRepo: TaskRepository,
    private val projectRepo: ProjectRepository,
    private val habitRepo: HabitRepository,
    private val calendarEventRepo: CalendarEventRepository,
    private val contactRepo: ContactRepository,
    private val noteRepo: NoteRepository,
    private val syncMetadataRepo: SyncMetadataRepository,
) {
    suspend fun syncAll(): SyncResult {
        val errors = mutableListOf<String>()
        val conflicts = mutableListOf<SyncConflict>()
        var tasksUp = 0; var tasksDown = 0; var tasksDel = 0
        var eventsDown = 0; var contactsSynced = 0; var notesSynced = 0

        // Sync CalDAV tasks
        if (credentialStore.hasCredentials("calendars")) {
            try {
                val result = syncTasks()
                tasksUp += result.uploaded
                tasksDown += result.downloaded
                tasksDel += result.deleted
                conflicts.addAll(result.conflicts)
                // Reconstruct projects and habits from synced tasks
                reconstructProjects()
                reconstructHabits()
            } catch (e: Exception) {
                errors.add("Task sync: ${e.message}")
            }

            try {
                eventsDown = syncEvents()
            } catch (e: Exception) {
                errors.add("Event sync: ${e.message}")
            }
        }

        // Sync CardDAV contacts
        if (credentialStore.hasCredentials("contacts")) {
            try {
                contactsSynced = syncContacts()
            } catch (e: Exception) {
                errors.add("Contact sync: ${e.message}")
            }
        }

        // Sync WebDAV notes
        if (credentialStore.hasCredentials("notes")) {
            try {
                notesSynced = syncNotes()
            } catch (e: Exception) {
                errors.add("Notes sync: ${e.message}")
            }
        }

        return SyncResult(tasksUp, tasksDown, tasksDel, eventsDown, contactsSynced, notesSynced, conflicts = conflicts, errors = errors)
    }

    private data class TaskSyncResult(
        val uploaded: Int = 0,
        val downloaded: Int = 0,
        val deleted: Int = 0,
        val conflicts: List<SyncConflict> = emptyList(),
    )

    private suspend fun syncTasks(): TaskSyncResult {
        val url = credentialStore.getServerUrl("calendars")!!
        val user = credentialStore.getUsername("calendars")!!
        val pass = credentialStore.getPassword("calendars")!!
        val client = CalDavClient(url, user, pass)

        try {
            val calendars = client.discoverCalendars().getOrThrow()
            val taskCalendar = calendars.firstOrNull { it.supportsVtodo }
                ?: return TaskSyncResult()

            var uploaded = 0
            var downloaded = 0
            var deleted = 0
            val conflicts = mutableListOf<SyncConflict>()

            // Push dirty tasks
            val dirtyTasks = taskRepo.getDirty()
            for (task in dirtyTasks) {
                val ical = VtodoConverter.taskToVcalendar(task)
                val href = task.syncHref ?: VtodoConverter.vtodoHref(taskCalendar.href, task.syncUid ?: task.id.toString())
                val condition = if (task.syncHref == null) PutCondition.CREATE_ONLY else PutCondition.UNCONDITIONAL
                val etag = if (task.syncEtag != null) task.syncEtag else null

                val putResult = client.putVtodo(href, condition, etag, ical)
                val hash = VtodoConverter.taskContentHash(task)
                val newEtag = putResult.getOrNull()
                if (newEtag != null) {
                    taskRepo.markSynced(task.id, hash, newEtag, href)
                    uploaded++
                } else {
                    Log.w("LampSync", "PUT failed for ${task.title}, clearing dirty flag")
                    // Clear dirty flag to prevent infinite retry; next pull will reconcile
                    taskRepo.markSynced(task.id, hash, etag ?: "", href)
                }
            }

            // Push deletes
            val deletedTasks = taskRepo.getDeleted()
            for ((id, syncHref, syncEtag) in deletedTasks) {
                if (syncHref != null && syncEtag != null) {
                    client.deleteVtodo(syncHref, syncEtag).getOrNull()
                }
                // Actually delete locally after remote delete
                taskRepo.delete(java.util.UUID.fromString(id))
                deleted++
            }

            // Pull remote changes via sync-collection
            val metadata = syncMetadataRepo.getByHref(taskCalendar.href)
            val syncToken = metadata?.syncToken
            val localTaskCount = taskRepo.getAll().size

            // Use full listing when no sync token or DB is empty (stale token)
            val (changes, newToken) = if (syncToken != null && localTaskCount > 0) {
                try {
                    client.syncCollection(taskCalendar.href, syncToken).getOrThrow()
                } catch (_: SyncTokenExpiredException) {
                    val vtodos = client.listVtodos(taskCalendar.href).getOrThrow()
                    vtodos.map { SyncChange.Changed(it) } to null
                }
            } else {
                val vtodos = client.listVtodos(taskCalendar.href).getOrThrow()
                vtodos.map { SyncChange.Changed(it) } to null
            }

            for (change in changes) {
                when (change) {
                    is SyncChange.Changed -> {
                        val remoteTask = VtodoConverter.vcalendarToTask(change.vtodo.icalBody)
                            ?: continue
                        val remoteWithSync = remoteTask.copy(
                            syncHref = change.vtodo.href,
                            syncEtag = change.vtodo.etag,
                            syncHash = VtodoConverter.taskContentHash(remoteTask),
                        )

                        val localTask = taskRepo.getByHref(change.vtodo.href)
                        if (localTask != null) {
                            // Check if local was modified
                            val localHash = VtodoConverter.taskContentHash(localTask)
                            if (localHash == localTask.syncHash) {
                                // Local unchanged, take remote
                                taskRepo.save(remoteWithSync.copy(id = localTask.id), markDirty = false)
                            } else {
                                // Both changed: produce conflict for user resolution
                                conflicts.add(SyncConflict.StateMismatch(
                                    localTask = localTask,
                                    remoteTask = remoteWithSync,
                                ))
                            }
                        } else {
                            // New remote task
                            taskRepo.save(remoteWithSync, markDirty = false)
                        }
                        downloaded++
                    }
                    is SyncChange.Deleted -> {
                        val local = taskRepo.getByHref(change.href)
                        if (local != null) {
                            // Local exists but remote deleted — surface as conflict
                            val localHash = VtodoConverter.taskContentHash(local)
                            if (localHash != local.syncHash) {
                                // Local was modified, let user decide
                                conflicts.add(SyncConflict.LocalOnly(localTask = local))
                            } else {
                                taskRepo.delete(local.id)
                            }
                        }
                        deleted++
                    }
                }
            }

            // Save new sync token
            if (newToken != null) {
                syncMetadataRepo.save(SyncMetadata(
                    calendarHref = taskCalendar.href,
                    syncToken = newToken,
                    lastSyncTimestamp = System.currentTimeMillis(),
                ))
            }

            return TaskSyncResult(uploaded, downloaded, deleted, conflicts)
        } finally {
            client.close()
        }
    }

    private suspend fun reconstructProjects() {
        val allTasks = taskRepo.getAll()
        val projectNames = allTasks.mapNotNull { it.project }.filter { it.isNotBlank() }.toSet()
        val existingProjects = projectRepo.getAll()
        val existingNames = existingProjects.map { it.name }.toSet()
        for (name in projectNames) {
            if (name !in existingNames) {
                projectRepo.save(Project(name = name))
            }
        }
    }

    private suspend fun reconstructHabits() {
        val allTasks = taskRepo.getAll()
        // Identify habits: has recurrence, logbook entries, or "habit" tag
        val habitTasks = allTasks.filter {
            it.recurrence != null || it.logbookEntries.isNotEmpty() || "habit" in it.extraTags
        }
        Log.d("LampSync", "reconstructHabits: ${allTasks.size} tasks, ${habitTasks.size} habits")
        for (task in habitTasks) {
            val existing = habitRepo.getByTaskId(task.id)
            if (existing == null) {
                Log.d("LampSync", "Creating habit for: ${task.title}")
                val habit = Habit(task = task, completions = task.logbookEntries)
                    .recalculateStreak(java.time.LocalDate.now())
                habitRepo.saveHabitOnly(habit)
            } else if (task.logbookEntries.isNotEmpty() && existing.completions != task.logbookEntries) {
                // Update completions from synced logbook entries
                Log.d("LampSync", "Updating habit completions for: ${task.title}")
                val updated = existing.copy(task = task, completions = task.logbookEntries)
                    .recalculateStreak(java.time.LocalDate.now())
                habitRepo.saveHabitOnly(updated)
            }
        }
    }

    private suspend fun syncEvents(): Int {
        val url = credentialStore.getServerUrl("calendars")!!
        val user = credentialStore.getUsername("calendars")!!
        val pass = credentialStore.getPassword("calendars")!!
        val client = CalDavClient(url, user, pass)

        try {
            val calendars = client.discoverCalendars().getOrThrow()
            var downloaded = 0

            for (calendar in calendars.filter { it.supportsVevent }) {
                val vevents = client.listVevents(calendar.href).getOrThrow()
                for (vevent in vevents) {
                    val event = VeventConverter.vcalendarToEvent(
                        vevent.icalBody, calendar.href, calendar.displayName
                    ) ?: continue
                    calendarEventRepo.save(event.copy(
                        syncHref = vevent.href,
                        syncHash = event.syncHash,
                        syncEtag = vevent.etag,
                    ), markDirty = false)
                    downloaded++
                }
            }

            return downloaded
        } finally {
            client.close()
        }
    }

    private suspend fun syncContacts(): Int {
        val url = credentialStore.getServerUrl("contacts")!!
        val user = credentialStore.getUsername("contacts")!!
        val pass = credentialStore.getPassword("contacts")!!
        val client = CardDavClient(url, user, pass)

        try {
            var synced = 0

            // 1. Push dirty contacts to CardDAV
            val dirtyContacts = contactRepo.getDirty()
            var addressbookHref: String? = null

            for (contact in dirtyContacts) {
                val vcard = CardDavClient.toVcard(contact)
                if (contact.syncHref != null) {
                    // Update existing
                    val newEtag = client.putContact(contact.syncHref!!, vcard, contact.syncEtag).getOrNull()
                        ?: continue
                    contactRepo.markSynced(contact.id, contact.syncHref!!, newEtag)
                } else {
                    // Create new — discover addressbook on first use
                    if (addressbookHref == null) {
                        addressbookHref = client.findFirstAddressbook().getOrNull()
                    }
                    val abHref = addressbookHref ?: continue
                    val (href, etag) = client.createContact(abHref, vcard, contact.id.toString()).getOrNull()
                        ?: continue
                    contactRepo.markSynced(contact.id, href, etag)
                }
                synced++
            }

            // 2. Push deleted contacts to CardDAV
            val deletedContacts = contactRepo.getDeleted()
            for (contact in deletedContacts) {
                if (contact.syncHref != null) {
                    val deleted = client.deleteContact(contact.syncHref!!).isSuccess
                    if (!deleted) continue // Keep locally until remote delete succeeds
                }
                contactRepo.delete(contact.id)
                synced++
            }

            // 3. Pull remote contacts and merge
            val remoteContacts = client.fetchContacts().getOrThrow()
            val localContacts = contactRepo.getAll()
            val merged = CardDavClient.mergeContacts(localContacts, remoteContacts)

            // Save merged contacts without marking dirty
            for (contact in merged) {
                contactRepo.save(contact, markDirty = false)
            }
            synced += merged.size

            return synced
        } finally {
            client.close()
        }
    }

    private suspend fun syncNotes(): Int {
        val url = credentialStore.getServerUrl("notes")!!
        val user = credentialStore.getUsername("notes")!!
        val pass = credentialStore.getPassword("notes")!!
        val client = WebDavClient(url, user, pass)

        try {
            client.ensureCollection().getOrThrow()
            val remoteFiles = client.listFiles().getOrThrow()
            val localNotes = noteRepo.getAll()
            val localByFilename = localNotes.associateBy { "${it.id}.org" }
            var synced = 0

            // Pull from remote
            for (file in remoteFiles) {
                val localNote = localByFilename[file.filename]
                if (localNote != null && localNote.syncEtag == file.etag) {
                    continue // No change
                }
                val (content, etag) = client.getFile(file.filename).getOrThrow()
                // Simple: store raw content as note body
                if (localNote != null) {
                    noteRepo.save(localNote.copy(body = content, syncEtag = etag))
                } else {
                    val id = try {
                        java.util.UUID.fromString(file.filename.removeSuffix(".org"))
                    } catch (_: Exception) { java.util.UUID.randomUUID() }
                    noteRepo.save(com.lamp.mobile.core.model.Note(
                        id = id,
                        title = file.filename.removeSuffix(".org"),
                        body = content,
                        syncEtag = etag,
                    ))
                }
                synced++
            }

            // Push local notes without etag
            for (note in localNotes) {
                if (note.syncEtag == null) {
                    val etag = client.putFile("${note.id}.org", note.body).getOrNull()
                    noteRepo.save(note.copy(syncEtag = etag))
                    synced++
                }
            }

            return synced
        } finally {
            client.close()
        }
    }
}
