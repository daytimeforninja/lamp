package com.lamp.mobile.core.database.entity

import androidx.room.Entity
import androidx.room.PrimaryKey

@Entity(tableName = "notes")
data class NoteEntity(
    @PrimaryKey val id: String,
    val title: String,
    val body: String,
    val tags: String, // JSON array
    val links: String, // JSON array of "type:uuid"
    val source: String?,
    val created: String, // ISO datetime
    val modified: String, // ISO datetime
    val syncEtag: String?,
)
