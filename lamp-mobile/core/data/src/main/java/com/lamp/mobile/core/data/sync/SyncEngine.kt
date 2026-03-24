package com.lamp.mobile.core.data.sync

import com.lamp.mobile.core.data.credential.CredentialStore
import com.lamp.mobile.core.data.repository.*
import java.time.LocalDate
import com.lamp.mobile.core.model.Habit
import com.lamp.mobile.core.model.ListKind
import com.lamp.mobile.core.model.Project
import com.lamp.mobile.core.model.SyncConflict
import com.lamp.mobile.core.model.SyncMetadata
import com.lamp.mobile.core.model.Task
import com.lamp.mobile.core.network.caldav.CalDavClient
import com.lamp.mobile.core.network.caldav.PutCondition
import com.lamp.mobile.core.network.caldav.SyncChange
import com.lamp.mobile.core.network.caldav.SyncTokenExpiredException
import com.lamp.mobile.core.network.carddav.CardDavClient
import com.lamp.mobile.core.network.imap.ImapClient
import com.lamp.mobile.core.network.imap.ImapEmail
import com.lamp.mobile.core.network.ical.VtodoConverter
import com.lamp.mobile.core.network.ical.VeventConverter
import com.lamp.mobile.core.network.org.OrgSerializer
import com.lamp.mobile.core.network.webdav.WebDavClient
import android.content.Context
import android.util.Log
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
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
    private val dayPlanRepo: DayPlanRepository,
    private val calendarEventRepo: CalendarEventRepository,
    private val contactRepo: ContactRepository,
    private val noteRepo: NoteRepository,
    private val listItemRepo: ListItemRepository,
    private val accountRepo: AccountRepository,
    private val syncMetadataRepo: SyncMetadataRepository,
    @ApplicationContext private val context: Context,
) {
    private val _imapEmails = MutableStateFlow<List<ImapEmail>>(emptyList())
    val imapEmails: StateFlow<List<ImapEmail>> = _imapEmails.asStateFlow()

    private val archivedUids = mutableSetOf<Long>()
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
                // Reconstruct projects, habits, and day plan from synced tasks
                reconstructProjects()
                reconstructHabits()
                reconstructDayPlan()
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

        // Sync WebDAV notes + shopping + accounts
        if (credentialStore.hasCredentials("notes")) {
            try {
                notesSynced = syncNotes()
            } catch (e: Exception) {
                errors.add("Notes sync: ${e.message}")
            }
            try {
                syncShoppingAndAccounts()
            } catch (e: Exception) {
                errors.add("Shopping/accounts sync: ${e.message}")
            }
        }

        // Fetch IMAP emails
        if (credentialStore.hasCredentials("imap")) {
            try {
                fetchImapEmails()
            } catch (e: Exception) {
                errors.add("IMAP: ${e.message}")
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

            // Push dirty tasks — stamp dayplanBudget on the first confirmed task (matching desktop)
            val dirtyTasks = taskRepo.getDirty()
            val today = LocalDate.now()
            val dayPlan = dayPlanRepo.getByDate(today)
            var firstConfirmed = true
            for (task in dirtyTasks) {
                val stamped = if (task.dayplanDate == today && dayPlan != null) {
                    if (firstConfirmed) {
                        firstConfirmed = false
                        task.copy(dayplanBudget = dayPlan.spoonBudget)
                    } else {
                        task.copy(dayplanBudget = null)
                    }
                } else {
                    task
                }
                val ical = VtodoConverter.taskToVcalendar(stamped)
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
                    Log.e("LampSync", "PUT failed for ${task.title}, will retry next sync")
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
                    Log.w("LampSync", "Sync token expired, doing full listing")
                    val vtodos = client.listVtodos(taskCalendar.href).getOrThrow()
                    // Get a fresh sync token for next time
                    val freshToken = client.getSyncToken(taskCalendar.href)
                    vtodos.map { SyncChange.Changed(it) } to freshToken
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
                            // Merge local fields that remote may not have
                            val merged = remoteWithSync.copy(
                                id = localTask.id,
                                project = remoteWithSync.project ?: localTask.project,
                                extraTags = if (remoteWithSync.extraTags.isEmpty()) localTask.extraTags
                                            else (remoteWithSync.extraTags + localTask.extraTags).distinct(),
                                logbookEntries = (remoteWithSync.logbookEntries + localTask.logbookEntries).distinct().sorted(),
                                clockEntries = (remoteWithSync.clockEntries + localTask.clockEntries).distinct(),
                            )
                            // Check if local was modified
                            val localHash = VtodoConverter.taskContentHash(localTask)
                            if (localHash == localTask.syncHash) {
                                // Local unchanged, take remote (with merged local fields)
                                taskRepo.save(merged, markDirty = false)
                            } else {
                                // Both changed: produce conflict for user resolution
                                conflicts.add(SyncConflict.StateMismatch(
                                    localTask = localTask,
                                    remoteTask = merged,
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
            // Ensure habit tasks have location = "habits" (sync may have reset to "inbox")
            habitRepo.save(
                habitRepo.getByTaskId(task.id)?.let { existing ->
                    if (task.logbookEntries.isNotEmpty() && existing.completions != task.logbookEntries) {
                        Log.d("LampSync", "Merging habit completions for: ${task.title}")
                        val combined = (existing.completions + task.logbookEntries).distinct().sorted()
                        existing.copy(task = task, completions = combined)
                            .recalculateStreak(java.time.LocalDate.now())
                    } else {
                        existing.copy(task = task)
                    }
                } ?: run {
                    Log.d("LampSync", "Creating habit for: ${task.title}")
                    Habit(task = task, completions = task.logbookEntries)
                        .recalculateStreak(java.time.LocalDate.now())
                }
            )
        }
    }

    private suspend fun reconstructDayPlan() {
        val today = LocalDate.now()
        val allTasks = taskRepo.getAll()
        val dayplanTasks = allTasks.filter { it.dayplanDate == today }
        val dayplanTaskIds = dayplanTasks.map { it.id }
        if (dayplanTaskIds.isEmpty()) return

        val existing = dayPlanRepo.getByDate(today)
            ?: com.lamp.mobile.core.model.DayPlan(date = today)
        // Remote plan exists — use it as source of truth
        val mergedIds = dayplanTaskIds.distinct()
        // Extract budget from the first task that carries it (matching desktop behavior)
        val remoteBudget = dayplanTasks.firstNotNullOfOrNull { it.dayplanBudget }
        val updatedPlan = if (remoteBudget != null) {
            existing.copy(confirmedTaskIds = mergedIds, spoonBudget = remoteBudget)
        } else {
            existing.copy(confirmedTaskIds = mergedIds)
        }
        if (updatedPlan != existing) {
            Log.d("LampSync", "Reconstructed day plan: ${mergedIds.size} tasks, budget=${remoteBudget ?: existing.spoonBudget}")
            dayPlanRepo.save(updatedPlan)
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

            // 3. Pull remote contacts and merge (excluding just-deleted ones)
            val justDeletedHrefs = deletedContacts.mapNotNull { it.syncHref }.toSet()
            val remoteContacts = client.fetchContacts().getOrThrow()
                .filter { (href, _) -> href !in justDeletedHrefs }
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

            // Pull from remote (skip non-UUID filenames like shopping.org, accounts.org)
            for (file in remoteFiles) {
                val stem = file.filename.removeSuffix(".org")
                try { java.util.UUID.fromString(stem) } catch (_: Exception) { continue }
                val localNote = localByFilename[file.filename]
                if (localNote != null && localNote.syncEtag == file.etag) {
                    continue // No change
                }
                val (content, etag) = client.getFile(file.filename).getOrThrow()
                // Simple: store raw content as note body
                if (localNote != null) {
                    val updatedTitle = content.lines()
                        .firstOrNull { it.startsWith("#+TITLE:") }
                        ?.substringAfter("#+TITLE:")
                        ?.trim()
                        ?: localNote.title
                    noteRepo.save(localNote.copy(title = updatedTitle, body = content, syncEtag = etag))
                } else {
                    val id = try {
                        java.util.UUID.fromString(file.filename.removeSuffix(".org"))
                    } catch (_: Exception) { java.util.UUID.randomUUID() }
                    val title = content.lines()
                        .firstOrNull { it.startsWith("#+TITLE:") }
                        ?.substringAfter("#+TITLE:")
                        ?.trim()
                        ?: file.filename.removeSuffix(".org")
                    noteRepo.save(com.lamp.mobile.core.model.Note(
                        id = id,
                        title = title,
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

    private suspend fun syncShoppingAndAccounts() {
        val url = credentialStore.getServerUrl("notes")!!
        val user = credentialStore.getUsername("notes")!!
        val pass = credentialStore.getPassword("notes")!!
        val client = WebDavClient(url, user, pass)

        try {
            client.ensureCollection().getOrThrow()

            // Sync shopping items
            val localShopping = listItemRepo.getByKind(ListKind.SHOPPING)
            val remoteShopping = try {
                val (content, _) = client.getFile("shopping.org").getOrThrow()
                Log.d("SyncEngine", "GET shopping.org (${content.length} bytes): ${content.take(200)}")
                OrgSerializer.parseShoppingItems(content)
            } catch (e: Exception) {
                Log.d("SyncEngine", "GET shopping.org failed: ${e.message}")
                emptyList()
            }

            // Merge: remote wins for matching UUIDs, keep local-only items
            val remoteIds = remoteShopping.map { it.id }.toSet()
            val merged = remoteShopping.toMutableList()
            for (item in localShopping) {
                if (item.id !in remoteIds) {
                    merged.add(item)
                }
            }

            // Save merged items locally
            val existingIds = localShopping.map { it.id }.toSet()
            for (item in merged) {
                listItemRepo.save(item)
            }
            // Remove local items that were deleted remotely
            val mergedIds = merged.map { it.id }.toSet()
            for (item in localShopping) {
                if (item.id !in mergedIds) {
                    listItemRepo.delete(item.id)
                }
            }

            // Push merged result if content differs from remote
            if (merged.isNotEmpty()) {
                val mergedContent = OrgSerializer.writeShoppingFile(merged)
                val remoteContent = OrgSerializer.writeShoppingFile(remoteShopping)
                if (mergedContent != remoteContent) {
                    Log.d("SyncEngine", "Pushing shopping.org (${mergedContent.length} bytes)")
                    client.putFile("shopping.org", mergedContent).getOrNull()
                }
            }

            Log.i("SyncEngine", "Shopping sync: ${remoteShopping.size} remote, ${localShopping.size} local, ${merged.size} merged")

            // Sync accounts
            val localAccounts = accountRepo.getAll()
            val remoteAccounts = try {
                val (content, _) = client.getFile("accounts.org").getOrThrow()
                OrgSerializer.parseAccounts(content)
            } catch (_: Exception) { emptyList() }

            val remoteAccountIds = remoteAccounts.map { it.id }.toSet()
            val mergedAccounts = remoteAccounts.toMutableList()
            for (account in localAccounts) {
                if (account.id !in remoteAccountIds) {
                    mergedAccounts.add(account)
                }
            }

            for (account in mergedAccounts) {
                accountRepo.save(account)
            }
            val mergedAccountIds = mergedAccounts.map { it.id }.toSet()
            for (account in localAccounts) {
                if (account.id !in mergedAccountIds) {
                    accountRepo.delete(account.id)
                }
            }

            if (mergedAccounts.isNotEmpty()) {
                val mergedAccountsContent = OrgSerializer.writeAccountsFile(mergedAccounts)
                val remoteAccountsContent = OrgSerializer.writeAccountsFile(remoteAccounts)
                if (mergedAccountsContent != remoteAccountsContent) {
                    client.putFile("accounts.org", mergedAccountsContent).getOrNull()
                }
            }

            Log.i("SyncEngine", "Accounts sync: ${remoteAccounts.size} remote, ${mergedAccounts.size} merged")
        } finally {
            client.close()
        }
    }

    private suspend fun fetchImapEmails() {
        val host = credentialStore.getServerUrl("imap") ?: return
        val username = credentialStore.getUsername("imap") ?: return
        val password = credentialStore.getPassword("imap") ?: return
        val prefs = context.getSharedPreferences("lamp_settings", Context.MODE_PRIVATE)
        val folder = prefs.getString("imap_folder", null)?.ifBlank { "INBOX" } ?: "INBOX"

        val client = ImapClient(host, username, password)
        val emails = client.fetchEmails(folder).getOrThrow()
        _imapEmails.value = emails.filter { it.uid !in archivedUids }
        Log.i("SyncEngine", "IMAP: fetched ${emails.size} emails from $folder")
    }

    suspend fun createTaskFromEmail(email: ImapEmail) {
        val note = buildString {
            appendLine("From: ${email.from}")
            email.date?.let { appendLine("Date: $it") }
            if (email.bodyFull.isNotEmpty()) {
                appendLine()
                append(email.bodyFull)
            }
        }
        val task = Task(title = email.subject, notes = note)
        taskRepo.save(task)
        archiveImapEmail(email.uid)
    }

    suspend fun archiveImapEmail(uid: Long) {
        val host = credentialStore.getServerUrl("imap") ?: return
        val username = credentialStore.getUsername("imap") ?: return
        val password = credentialStore.getPassword("imap") ?: return
        val prefs = context.getSharedPreferences("lamp_settings", Context.MODE_PRIVATE)
        val folder = prefs.getString("imap_folder", null)?.ifBlank { "INBOX" } ?: "INBOX"

        val client = ImapClient(host, username, password)
        client.archiveEmail(folder, uid).getOrThrow()
        archivedUids.add(uid)
        _imapEmails.value = _imapEmails.value.filter { it.uid != uid }
    }
}
