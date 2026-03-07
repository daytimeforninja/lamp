package com.lamp.mobile.core.database.entity

import androidx.room.Entity
import androidx.room.PrimaryKey

@Entity(tableName = "contacts")
data class ContactEntity(
    @PrimaryKey val id: String,
    val name: String,
    val email: String?,
    val phone: String?,
    val website: String?,
    val signal: String?,
    val preferredMethod: String?,
    val category: String,
    val lastContacted: String?, // ISO date
    val syncHref: String?,
    val syncEtag: String?,
    val syncDirty: Boolean = false,
    val syncDeleted: Boolean = false,
)
