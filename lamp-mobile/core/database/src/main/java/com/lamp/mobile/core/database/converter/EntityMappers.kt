package com.lamp.mobile.core.database.converter

import com.lamp.mobile.core.database.entity.*
import com.lamp.mobile.core.model.*
import org.json.JSONArray
import org.json.JSONObject
import java.time.LocalDate
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter
import java.util.UUID

object EntityMappers {

    private val DATE_FMT = DateTimeFormatter.ISO_LOCAL_DATE
    private val DATETIME_FMT = DateTimeFormatter.ISO_LOCAL_DATE_TIME

    // --- JSON helpers ---

    private fun stringListToJson(list: List<String>): String =
        JSONArray(list).toString()

    private fun jsonToStringList(json: String): List<String> {
        if (json.isBlank()) return emptyList()
        return try {
            val arr = JSONArray(json)
            (0 until arr.length()).map { arr.getString(it) }
        } catch (_: Exception) { emptyList() }
    }

    private fun uuidListToJson(list: List<UUID>): String =
        JSONArray(list.map { it.toString() }).toString()

    private fun jsonToUuidList(json: String): List<UUID> {
        if (json.isBlank()) return emptyList()
        return try {
            val arr = JSONArray(json)
            (0 until arr.length()).mapNotNull { i ->
                try { UUID.fromString(arr.getString(i)) } catch (_: Exception) { null }
            }
        } catch (_: Exception) { emptyList() }
    }

    private fun datetimeListToJson(list: List<LocalDateTime>): String =
        JSONArray(list.map { it.format(DATETIME_FMT) }).toString()

    private fun jsonToDatetimeList(json: String): List<LocalDateTime> {
        if (json.isBlank()) return emptyList()
        return try {
            val arr = JSONArray(json)
            (0 until arr.length()).mapNotNull { i ->
                try { LocalDateTime.parse(arr.getString(i), DATETIME_FMT) } catch (_: Exception) { null }
            }
        } catch (_: Exception) { emptyList() }
    }

    private fun completedTasksToJson(tasks: List<CompletedTask>): String {
        val arr = JSONArray()
        tasks.forEach { t ->
            val obj = JSONObject()
            obj.put("id", t.id.toString())
            obj.put("title", t.title)
            obj.put("esc", t.esc ?: JSONObject.NULL)
            arr.put(obj)
        }
        return arr.toString()
    }

    private fun jsonToCompletedTasks(json: String): List<CompletedTask> {
        if (json.isBlank()) return emptyList()
        return try {
            val arr = JSONArray(json)
            (0 until arr.length()).mapNotNull { i ->
                try {
                    val obj = arr.getJSONObject(i)
                    CompletedTask(
                        id = UUID.fromString(obj.getString("id")),
                        title = obj.getString("title"),
                        esc = if (obj.isNull("esc")) null else obj.getInt("esc"),
                    )
                } catch (_: Exception) { null }
            }
        } catch (_: Exception) { emptyList() }
    }

    private fun linksToJson(links: List<LinkTarget>): String =
        JSONArray(links.map { it.toOrg() }).toString()

    private fun jsonToLinks(json: String): List<LinkTarget> {
        if (json.isBlank()) return emptyList()
        return try {
            val arr = JSONArray(json)
            (0 until arr.length()).mapNotNull { i ->
                try { LinkTarget.fromOrg(arr.getString(i)) } catch (_: Exception) { null }
            }
        } catch (_: Exception) { emptyList() }
    }

    // --- Task ---

    private fun parseDate(s: String?): LocalDate? =
        s?.let { try { LocalDate.parse(it, DATE_FMT) } catch (_: Exception) { null } }

    private fun parseDatetime(s: String?): LocalDateTime? =
        s?.let { try { LocalDateTime.parse(it, DATETIME_FMT) } catch (_: Exception) { null } }

    fun TaskEntity.toDomain(): Task = Task(
        id = UUID.fromString(id),
        title = title,
        state = TaskState.fromKeyword(state) ?: TaskState.TODO,
        priority = priority?.let { Priority.fromOrg(it) },
        contexts = jsonToStringList(contexts),
        scheduled = parseDate(scheduled),
        deadline = parseDate(deadline),
        recurrence = recurrence?.let { Recurrence.parse(it) },
        notes = notes,
        created = parseDatetime(created) ?: LocalDateTime.now(),
        completed = parseDatetime(completed),
        project = project,
        waitingFor = waitingFor,
        esc = esc,
        delegated = parseDate(delegated),
        followUp = parseDate(followUp),
        extraTags = jsonToStringList(extraTags),
        scheduledTime = scheduledTime,
        deadlineTime = deadlineTime,
        logbookEntries = jsonToDatetimeList(logbookEntries),
        syncHref = syncHref,
        syncHash = syncHash,
        syncUid = syncUid,
        syncEtag = syncEtag,
    )

    fun Task.toEntity(syncDirty: Boolean = false, syncDeleted: Boolean = false, location: String? = null): TaskEntity = TaskEntity(
        id = id.toString(),
        title = title,
        state = state.keyword,
        priority = priority?.orgString,
        contexts = stringListToJson(contexts),
        scheduled = scheduled?.format(DATE_FMT),
        deadline = deadline?.format(DATE_FMT),
        recurrence = recurrence?.toString(),
        notes = notes,
        created = created.format(DATETIME_FMT),
        completed = completed?.format(DATETIME_FMT),
        project = project,
        waitingFor = waitingFor,
        esc = esc,
        delegated = delegated?.format(DATE_FMT),
        followUp = followUp?.format(DATE_FMT),
        extraTags = stringListToJson(extraTags),
        scheduledTime = scheduledTime,
        deadlineTime = deadlineTime,
        logbookEntries = datetimeListToJson(logbookEntries),
        syncHref = syncHref,
        syncHash = syncHash,
        syncUid = syncUid,
        syncEtag = syncEtag,
        syncDirty = syncDirty,
        syncDeleted = syncDeleted,
        location = location,
    )

    // --- Project ---

    fun ProjectEntity.toDomain(tasks: List<Task> = emptyList()): Project = Project(
        id = UUID.fromString(id),
        name = name,
        tasks = tasks,
        purpose = purpose,
        outcome = outcome,
        brainstorm = brainstorm,
    )

    fun Project.toEntity(): ProjectEntity = ProjectEntity(
        id = id.toString(),
        name = name,
        purpose = purpose,
        outcome = outcome,
        brainstorm = brainstorm,
    )

    // --- Habit ---

    fun HabitEntity.toDomain(task: Task): Habit = Habit(
        task = task,
        completions = jsonToDatetimeList(completions),
        streak = streak,
        bestStreak = bestStreak,
    )

    fun Habit.toEntity(): HabitEntity = HabitEntity(
        taskId = task.id.toString(),
        completions = datetimeListToJson(completions),
        streak = streak,
        bestStreak = bestStreak,
    )

    // --- DayPlan ---

    fun DayPlanEntity.toDomain(): DayPlan = DayPlan(
        date = parseDate(date) ?: LocalDate.now(),
        spoonBudget = spoonBudget,
        activeContexts = jsonToStringList(activeContexts),
        confirmedTaskIds = jsonToUuidList(confirmedTaskIds),
        completedTasks = jsonToCompletedTasks(completedTasks),
        spentSpoons = spentSpoons,
        pickedMediaIds = jsonToUuidList(pickedMediaIds),
        pickedShoppingIds = jsonToUuidList(pickedShoppingIds),
    )

    fun DayPlan.toEntity(): DayPlanEntity = DayPlanEntity(
        date = date.format(DATE_FMT),
        spoonBudget = spoonBudget,
        activeContexts = stringListToJson(activeContexts),
        confirmedTaskIds = uuidListToJson(confirmedTaskIds),
        completedTasks = completedTasksToJson(completedTasks),
        spentSpoons = spentSpoons,
        pickedMediaIds = uuidListToJson(pickedMediaIds),
        pickedShoppingIds = uuidListToJson(pickedShoppingIds),
    )

    // --- CalendarEvent ---

    fun CalendarEventEntity.toDomain(): CalendarEvent = CalendarEvent(
        id = UUID.fromString(id),
        title = title,
        start = parseDatetime(start) ?: LocalDateTime.now(),
        end = parseDatetime(end) ?: LocalDateTime.now(),
        allDay = allDay,
        location = location,
        description = description,
        status = EventStatus.fromIcal(status),
        calendarHref = calendarHref,
        calendarName = calendarName,
        syncHref = syncHref,
        syncHash = syncHash,
        syncEtag = syncEtag,
        syncUid = syncUid,
    )

    fun CalendarEvent.toEntity(syncDirty: Boolean = false, syncDeleted: Boolean = false): CalendarEventEntity = CalendarEventEntity(
        id = id.toString(),
        title = title,
        start = start.format(DATETIME_FMT),
        end = end.format(DATETIME_FMT),
        allDay = allDay,
        location = location,
        description = description,
        status = status.name,
        calendarHref = calendarHref,
        calendarName = calendarName,
        syncHref = syncHref,
        syncHash = syncHash,
        syncEtag = syncEtag,
        syncDirty = syncDirty,
        syncDeleted = syncDeleted,
        syncUid = syncUid,
    )

    // --- Note ---

    fun NoteEntity.toDomain(): Note = Note(
        id = UUID.fromString(id),
        title = title,
        body = body,
        tags = jsonToStringList(tags),
        links = jsonToLinks(links),
        source = source,
        created = parseDatetime(created) ?: LocalDateTime.now(),
        modified = parseDatetime(modified) ?: LocalDateTime.now(),
        syncEtag = syncEtag,
    )

    fun Note.toEntity(): NoteEntity = NoteEntity(
        id = id.toString(),
        title = title,
        body = body,
        tags = stringListToJson(tags),
        links = linksToJson(links),
        source = source,
        created = created.format(DATETIME_FMT),
        modified = modified.format(DATETIME_FMT),
        syncEtag = syncEtag,
    )

    // --- Contact ---

    fun ContactEntity.toDomain(): Contact = Contact(
        id = UUID.fromString(id),
        name = name,
        email = email,
        phone = phone,
        website = website,
        signal = signal,
        preferredMethod = preferredMethod,
        category = ContactCategory.fromString(category),
        lastContacted = parseDate(lastContacted),
        syncHref = syncHref,
        syncEtag = syncEtag,
        syncDirty = syncDirty,
        syncDeleted = syncDeleted,
    )

    fun Contact.toEntity(syncDirty: Boolean = false, syncDeleted: Boolean = false): ContactEntity = ContactEntity(
        id = id.toString(),
        name = name,
        email = email,
        phone = phone,
        website = website,
        signal = signal,
        preferredMethod = preferredMethod,
        category = category.toString(),
        lastContacted = lastContacted?.format(DATE_FMT),
        syncHref = syncHref,
        syncEtag = syncEtag,
        syncDirty = syncDirty,
        syncDeleted = syncDeleted,
    )

    // --- ListItem ---

    fun ListItemEntity.toDomain(): ListItem = ListItem(
        id = UUID.fromString(id),
        title = title,
        notes = notes,
        created = parseDatetime(created) ?: LocalDateTime.now(),
        done = done,
        kind = try { ListKind.valueOf(kind) } catch (_: Exception) { ListKind.MEDIA },
    )

    fun ListItem.toEntity(): ListItemEntity = ListItemEntity(
        id = id.toString(),
        title = title,
        notes = notes,
        created = created.format(DATETIME_FMT),
        done = done,
        kind = kind.name,
    )

    // --- Account ---

    fun AccountEntity.toDomain(): Account = Account(
        id = UUID.fromString(id),
        name = name,
        url = url,
        notes = notes,
        lastChecked = parseDate(lastChecked),
    )

    fun Account.toEntity(): AccountEntity = AccountEntity(
        id = id.toString(),
        name = name,
        url = url,
        notes = notes,
        lastChecked = lastChecked?.format(DATE_FMT),
    )

    // --- SyncMetadata ---

    fun SyncMetadataEntity.toDomain(): SyncMetadata = SyncMetadata(
        calendarHref = calendarHref,
        syncToken = syncToken,
        lastSyncTimestamp = lastSyncTimestamp,
    )

    fun SyncMetadata.toEntity(): SyncMetadataEntity = SyncMetadataEntity(
        calendarHref = calendarHref,
        syncToken = syncToken,
        lastSyncTimestamp = lastSyncTimestamp,
    )
}
