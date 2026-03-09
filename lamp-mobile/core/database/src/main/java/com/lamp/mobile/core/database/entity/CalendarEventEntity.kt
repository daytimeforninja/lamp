package com.lamp.mobile.core.database.entity

import androidx.room.Entity
import androidx.room.PrimaryKey

@Entity(tableName = "calendar_events")
data class CalendarEventEntity(
    @PrimaryKey val id: String,
    val title: String,
    val start: String, // ISO datetime
    val end: String, // ISO datetime
    val allDay: Boolean,
    val location: String,
    val description: String,
    val status: String,
    val calendarHref: String,
    val calendarName: String,
    val syncHref: String?,
    val syncHash: Long?,
    val syncEtag: String? = null,
    val syncDirty: Boolean = false,
    val syncDeleted: Boolean = false,
    val syncUid: String? = null,
)
