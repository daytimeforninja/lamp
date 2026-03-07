package com.lamp.mobile.core.database.entity

import androidx.room.Entity
import androidx.room.PrimaryKey

@Entity(tableName = "list_items")
data class ListItemEntity(
    @PrimaryKey val id: String,
    val title: String,
    val notes: String,
    val created: String, // ISO datetime
    val done: Boolean,
    val kind: String, // "MEDIA" or "SHOPPING"
)
