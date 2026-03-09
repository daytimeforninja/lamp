package com.lamp.mobile.core.model

import java.time.LocalDateTime
import java.util.UUID

enum class EventStatus {
    CONFIRMED,
    TENTATIVE,
    CANCELLED;

    companion object {
        fun fromIcal(s: String): EventStatus = when (s.uppercase()) {
            "TENTATIVE" -> TENTATIVE
            "CANCELLED" -> CANCELLED
            else -> CONFIRMED
        }
    }
}

data class CalendarEvent(
    val id: UUID = UUID.randomUUID(),
    val title: String,
    val start: LocalDateTime,
    val end: LocalDateTime,
    val allDay: Boolean = false,
    val location: String = "",
    val description: String = "",
    val status: EventStatus = EventStatus.CONFIRMED,
    val calendarHref: String = "",
    val calendarName: String = "",
    val syncHref: String? = null,
    val syncHash: Long? = null,
    val syncEtag: String? = null,
    val syncUid: String? = null,
)
