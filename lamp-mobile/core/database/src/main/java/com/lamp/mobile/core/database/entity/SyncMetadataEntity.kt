package com.lamp.mobile.core.database.entity

import androidx.room.Entity
import androidx.room.PrimaryKey

@Entity(tableName = "sync_metadata")
data class SyncMetadataEntity(
    @PrimaryKey val calendarHref: String,
    val syncToken: String?,
    val lastSyncTimestamp: Long,
)
