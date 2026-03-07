package com.lamp.mobile.core.model

/**
 * Tracks per-calendar sync tokens for incremental CalDAV sync.
 */
data class SyncMetadata(
    val calendarHref: String,
    val syncToken: String? = null,
    val lastSyncTimestamp: Long = 0L,
)
