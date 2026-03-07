package com.lamp.mobile.core.database.entity

import androidx.room.Entity
import androidx.room.PrimaryKey

@Entity(tableName = "tasks")
data class TaskEntity(
    @PrimaryKey val id: String,
    val title: String,
    val state: String,
    val priority: String?,
    val contexts: String, // JSON array
    val scheduled: String?, // ISO date
    val deadline: String?, // ISO date
    val recurrence: String?, // org format e.g. "+1w"
    val notes: String,
    val created: String, // ISO datetime
    val completed: String?, // ISO datetime
    val project: String?,
    val waitingFor: String?,
    val esc: Int?,
    val delegated: String?, // ISO date
    val followUp: String?, // ISO date
    val extraTags: String, // JSON array
    val scheduledTime: String?,
    val deadlineTime: String?,
    val logbookEntries: String, // JSON array of ISO datetimes
    val syncHref: String?,
    val syncHash: Long?,
    val syncUid: String?,
    val syncEtag: String?,
    val syncDirty: Boolean = false,
    val syncDeleted: Boolean = false,
    val location: String?, // which org file: inbox, next, waiting, someday
)
