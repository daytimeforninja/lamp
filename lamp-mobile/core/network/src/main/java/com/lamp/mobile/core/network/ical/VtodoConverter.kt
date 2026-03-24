package com.lamp.mobile.core.network.ical

import com.lamp.mobile.core.model.*
import java.time.LocalDate
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter
import java.util.UUID

/**
 * VTODO <-> Task conversion matching desktop's src/sync/vtodo.rs exactly.
 * Custom X-LAMP-* properties preserve GTD-specific fields.
 */
object VtodoConverter {

    private val CALDAV_UUID_NAMESPACE = ICalHelpers.CALDAV_UUID_NAMESPACE

    fun taskToVcalendar(task: Task): String {
        val lines = mutableListOf<String>()
        lines.add("BEGIN:VCALENDAR")
        lines.add("VERSION:2.0")
        lines.add("PRODID:-//Lamp GTD//EN")
        lines.add("BEGIN:VTODO")

        // UID: prefer sync_uid, fallback to extracting from href, fallback to task id
        val uid = task.syncUid
            ?: task.syncHref?.substringAfterLast('/')?.removeSuffix(".ics")
            ?: task.id.toString()
        lines.add(ICalHelpers.foldLine("UID:$uid"))

        // SUMMARY
        lines.add(ICalHelpers.foldLine("SUMMARY:${ICalHelpers.escapeText(task.title)}"))

        // STATUS
        val status = when (task.state) {
            TaskState.DONE -> "COMPLETED"
            TaskState.CANCELLED -> "CANCELLED"
            else -> "NEEDS-ACTION"
        }
        lines.add("STATUS:$status")

        // PERCENT-COMPLETE for done tasks
        if (task.state.isDone) {
            lines.add("PERCENT-COMPLETE:100")
        }

        // PRIORITY: A=1, B=5, C=9, None=0
        val priority = when (task.priority) {
            Priority.A -> 1
            Priority.B -> 5
            Priority.C -> 9
            null -> 0
        }
        lines.add("PRIORITY:$priority")

        // CATEGORIES
        if (task.contexts.isNotEmpty()) {
            lines.add(ICalHelpers.foldLine("CATEGORIES:${task.contexts.joinToString(",")}"))
        }

        // DTSTART
        task.scheduled?.let { lines.add("DTSTART;VALUE=DATE:${ICalHelpers.formatDate(it)}") }

        // DUE
        task.deadline?.let { lines.add("DUE;VALUE=DATE:${ICalHelpers.formatDate(it)}") }

        // DESCRIPTION
        if (task.notes.isNotBlank()) {
            lines.add(ICalHelpers.foldLine("DESCRIPTION:${ICalHelpers.escapeText(task.notes)}"))
        }

        // CREATED
        lines.add("CREATED:${ICalHelpers.formatDatetime(task.created)}")

        // COMPLETED
        task.completed?.let { lines.add("COMPLETED:${ICalHelpers.formatDatetime(it)}") }

        // LAST-MODIFIED
        lines.add("LAST-MODIFIED:${ICalHelpers.formatDatetime(LocalDateTime.now())}")

        // X-LAMP-STATE
        lines.add("X-LAMP-STATE:${task.state.keyword}")

        // X-LAMP-PROJECT
        task.project?.let { lines.add(ICalHelpers.foldLine("X-LAMP-PROJECT:$it")) }

        // X-LAMP-WAITING-FOR
        task.waitingFor?.let { lines.add(ICalHelpers.foldLine("X-LAMP-WAITING-FOR:$it")) }

        // X-LAMP-ESC
        task.esc?.let { lines.add("X-LAMP-ESC:$it") }

        // X-LAMP-DELEGATED
        task.delegated?.let { lines.add("X-LAMP-DELEGATED:${ICalHelpers.formatDate(it)}") }

        // X-LAMP-FOLLOW-UP
        task.followUp?.let { lines.add("X-LAMP-FOLLOW-UP:${ICalHelpers.formatDate(it)}") }

        // X-LAMP-RECURRENCE
        task.recurrence?.let { lines.add("X-LAMP-RECURRENCE:$it") }

        // X-LAMP-DAYPLAN
        task.dayplanDate?.let {
            val dayplanLine = if (task.dayplanBudget != null) {
                "X-LAMP-DAYPLAN:${ICalHelpers.formatDate(it)};BUDGET=${task.dayplanBudget}"
            } else {
                "X-LAMP-DAYPLAN:${ICalHelpers.formatDate(it)}"
            }
            lines.add(dayplanLine)
        }

        // X-LAMP-TAGS (non-context tags like "habit")
        if (task.extraTags.isNotEmpty()) {
            lines.add(ICalHelpers.foldLine("X-LAMP-TAGS:${task.extraTags.joinToString(",") { ICalHelpers.escapeText(it) }}"))
        }

        // X-LAMP-LOGBOOK (habit completion timestamps)
        if (task.logbookEntries.isNotEmpty()) {
            val entries = task.logbookEntries.joinToString(",") { ICalHelpers.formatDatetime(it) }
            lines.add(ICalHelpers.foldLine("X-LAMP-LOGBOOK:$entries"))
        }

        // X-LAMP-CLOCK (work timer sessions)
        if (task.clockEntries.isNotEmpty()) {
            val entries = task.clockEntries.joinToString(",") { (s, e) ->
                "${ICalHelpers.formatDatetime(s)}/${ICalHelpers.formatDatetime(e)}"
            }
            lines.add(ICalHelpers.foldLine("X-LAMP-CLOCK:$entries"))
        }

        lines.add("END:VTODO")
        lines.add("END:VCALENDAR")

        return lines.joinToString("\r\n")
    }

    fun vcalendarToTask(ical: String): Task? {
        val unfolded = ICalHelpers.unfoldLines(ical)
        val lines = unfolded.lines()

        var inVtodo = false
        var uid: String? = null
        var summary: String? = null
        var status: String? = null
        var priority = 0
        var categories = emptyList<String>()
        var dtstart: LocalDate? = null
        var due: LocalDate? = null
        var description: String? = null
        var created: LocalDateTime? = null
        var completed: LocalDateTime? = null
        var lampState: String? = null
        var lampProject: String? = null
        var lampWaitingFor: String? = null
        var lampEsc: Int? = null
        var lampDelegated: LocalDate? = null
        var lampFollowUp: LocalDate? = null
        var lampRecurrence: String? = null
        var lampLogbook: String? = null
        var lampClock: String? = null
        var lampTags: String? = null
        var lampDayplan: LocalDate? = null
        var dayplanBudget: Int? = null

        for (line in lines) {
            val trimmed = line.trim()
            if (trimmed == "BEGIN:VTODO") { inVtodo = true; continue }
            if (trimmed == "END:VTODO") break
            if (!inVtodo) continue

            val parsed = ICalHelpers.parseIcalLine(trimmed) ?: continue
            val (key, value) = parsed

            when (key.uppercase()) {
                "UID" -> uid = value
                "SUMMARY" -> summary = ICalHelpers.unescapeText(value)
                "STATUS" -> status = value.uppercase()
                "PRIORITY" -> priority = value.toIntOrNull() ?: 0
                "CATEGORIES" -> categories = value.split(",").map { it.trim() }.filter { it.isNotEmpty() }
                "DTSTART" -> dtstart = ICalHelpers.parseIcalDate(value)
                "DUE" -> due = ICalHelpers.parseIcalDate(value)
                "DESCRIPTION" -> description = ICalHelpers.unescapeText(value)
                "CREATED" -> created = ICalHelpers.parseIcalDatetime(value)
                "COMPLETED" -> completed = ICalHelpers.parseIcalDatetime(value)
                "X-LAMP-STATE" -> lampState = value
                "X-LAMP-PROJECT" -> lampProject = value
                "X-LAMP-WAITING-FOR" -> lampWaitingFor = value
                "X-LAMP-ESC" -> lampEsc = value.toIntOrNull()
                "X-LAMP-DELEGATED" -> lampDelegated = ICalHelpers.parseIcalDate(value)
                "X-LAMP-FOLLOW-UP" -> lampFollowUp = ICalHelpers.parseIcalDate(value)
                "X-LAMP-RECURRENCE" -> lampRecurrence = value
                "X-LAMP-LOGBOOK" -> lampLogbook = value
                "X-LAMP-CLOCK" -> lampClock = value
                "X-LAMP-TAGS" -> lampTags = value
                "X-LAMP-DAYPLAN" -> {
                    if (';' in value) {
                        val parts = value.split(";", limit = 2)
                        lampDayplan = ICalHelpers.parseIcalDate(parts[0])
                        val budgetPart = parts[1]
                        if (budgetPart.startsWith("BUDGET=")) {
                            dayplanBudget = budgetPart.removePrefix("BUDGET=").toIntOrNull()
                        }
                    } else {
                        lampDayplan = ICalHelpers.parseIcalDate(value)
                    }
                }
            }
        }

        if (summary.isNullOrBlank()) return null

        // UID -> UUID: try parse, fallback to UUID v5
        val taskId = uid?.let {
            try { UUID.fromString(it) }
            catch (_: Exception) { ICalHelpers.uuidV5(CALDAV_UUID_NAMESPACE, it) }
        } ?: UUID.randomUUID()

        // State: X-LAMP-STATE takes priority over STATUS
        val taskState = lampState?.let { TaskState.fromKeyword(it) }
            ?: when (status) {
                "COMPLETED" -> TaskState.DONE
                "CANCELLED" -> TaskState.CANCELLED
                else -> TaskState.TODO
            }

        // Priority: 1->A, 2-5->B, 6-9->C, 0->null
        val taskPriority = when (priority) {
            1 -> Priority.A
            in 2..5 -> Priority.B
            in 6..9 -> Priority.C
            else -> null
        }

        return Task(
            id = taskId,
            title = summary,
            state = taskState,
            priority = taskPriority,
            contexts = categories,
            scheduled = dtstart,
            deadline = due,
            recurrence = lampRecurrence?.let { Recurrence.parse(it) },
            notes = description ?: "",
            created = created ?: LocalDateTime.now(),
            completed = completed,
            project = lampProject,
            waitingFor = lampWaitingFor,
            esc = lampEsc,
            delegated = lampDelegated,
            followUp = lampFollowUp,
            extraTags = lampTags
                ?.split(",")
                ?.map { ICalHelpers.unescapeText(it.trim()) }
                ?.filter { it.isNotEmpty() }
                ?: emptyList(),
            logbookEntries = lampLogbook
                ?.split(",")
                ?.mapNotNull { ICalHelpers.parseIcalDatetime(it.trim()) }
                ?: emptyList(),
            clockEntries = lampClock
                ?.split(",")
                ?.mapNotNull { pair ->
                    val parts = pair.trim().split("/", limit = 2)
                    if (parts.size == 2) {
                        val start = ICalHelpers.parseIcalDatetime(parts[0]) ?: return@mapNotNull null
                        val end = ICalHelpers.parseIcalDatetime(parts[1]) ?: return@mapNotNull null
                        start to end
                    } else null
                }
                ?: emptyList(),
            dayplanDate = lampDayplan,
            dayplanBudget = dayplanBudget,
            syncUid = uid,
        )
    }

    /**
     * Content hash matching desktop's vtodo.rs:task_content_hash().
     * Uses FNV-1a over fields in the same order as the Rust implementation.
     */
    fun taskContentHash(task: Task): Long {
        val hasher = StableHasher()
        hasher.writeString(task.title)
        hasher.writeString(task.state.keyword)
        hasher.writeOptionalString(task.priority?.orgString)
        // contexts: hash each individually
        for (ctx in task.contexts) {
            hasher.writeString(ctx)
        }
        hasher.writeOptionalString(task.scheduled?.format(DateTimeFormatter.ISO_LOCAL_DATE))
        hasher.writeOptionalString(task.deadline?.format(DateTimeFormatter.ISO_LOCAL_DATE))
        hasher.writeString(task.notes)
        hasher.writeOptionalString(task.project)
        hasher.writeOptionalString(task.waitingFor)
        hasher.writeOptionalInt(task.esc)
        hasher.writeOptionalString(task.delegated?.format(DateTimeFormatter.ISO_LOCAL_DATE))
        hasher.writeOptionalString(task.followUp?.format(DateTimeFormatter.ISO_LOCAL_DATE))
        hasher.writeOptionalString(task.recurrence?.toString())
        for (tag in task.extraTags) {
            hasher.writeString(tag)
        }
        for (entry in task.logbookEntries) {
            hasher.writeString(entry.format(DateTimeFormatter.ISO_LOCAL_DATE_TIME))
        }
        for ((start, end) in task.clockEntries) {
            hasher.writeString(start.format(DateTimeFormatter.ISO_LOCAL_DATE_TIME))
            hasher.writeString(end.format(DateTimeFormatter.ISO_LOCAL_DATE_TIME))
        }
        // Include dayplanDate so confirming/unconfirming a task triggers sync push
        hasher.writeOptionalString(task.dayplanDate?.format(DateTimeFormatter.ISO_LOCAL_DATE))
        hasher.writeOptionalInt(task.dayplanBudget)
        return hasher.finish()
    }

    /** Build the href for a task in a calendar collection */
    fun vtodoHref(calendarHref: String, taskUid: String): String =
        "${calendarHref.trimEnd('/')}/$taskUid.ics"
}
