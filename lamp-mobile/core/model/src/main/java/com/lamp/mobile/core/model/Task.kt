package com.lamp.mobile.core.model

import java.time.LocalDate
import java.time.LocalDateTime
import java.util.UUID

data class Task(
    val id: UUID = UUID.randomUUID(),
    val title: String,
    val state: TaskState = TaskState.TODO,
    val priority: Priority? = null,
    val contexts: List<String> = emptyList(),
    val scheduled: LocalDate? = null,
    val deadline: LocalDate? = null,
    val recurrence: Recurrence? = null,
    val notes: String = "",
    val created: LocalDateTime = LocalDateTime.now(),
    val completed: LocalDateTime? = null,
    val project: String? = null,
    val waitingFor: String? = null,
    val esc: Int? = null,
    val delegated: LocalDate? = null,
    val followUp: LocalDate? = null,
    val extraTags: List<String> = emptyList(),
    val scheduledTime: String? = null,
    val deadlineTime: String? = null,
    val logbookEntries: List<LocalDateTime> = emptyList(),
    val dayplanDate: LocalDate? = null,
    val syncHref: String? = null,
    val syncHash: Long? = null,
    val syncUid: String? = null,
    val syncEtag: String? = null,
) {
    fun complete(): Task = copy(
        state = TaskState.DONE,
        completed = LocalDateTime.now(),
    )

    fun cancel(): Task = copy(
        state = TaskState.CANCELLED,
        completed = LocalDateTime.now(),
    )

    fun isToday(today: LocalDate): Boolean {
        if (state.isDone) return false
        val scheduledToday = scheduled?.let { !it.isAfter(today) } ?: false
        val deadlineSoon = deadline?.let { !it.isAfter(today.plusDays(7)) } ?: false
        return scheduledToday || deadlineSoon
    }

    fun hasContext(ctx: String): Boolean = contexts.contains(ctx)
}
