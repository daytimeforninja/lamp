package com.lamp.mobile.core.model

import java.time.LocalDate
import java.util.UUID

enum class ContactCategory {
    PERSONAL,
    SERVICE;

    override fun toString(): String = when (this) {
        PERSONAL -> "Personal"
        SERVICE -> "Service"
    }

    companion object {
        fun fromString(s: String): ContactCategory = when (s.lowercase()) {
            "service" -> SERVICE
            else -> PERSONAL
        }
    }
}

data class Contact(
    val id: UUID = UUID.randomUUID(),
    val name: String,
    val email: String? = null,
    val phone: String? = null,
    val website: String? = null,
    val signal: String? = null,
    val preferredMethod: String? = null,
    val category: ContactCategory = ContactCategory.PERSONAL,
    val lastContacted: LocalDate? = null,
    val syncHref: String? = null,
    val syncEtag: String? = null,
    val syncDirty: Boolean = false,
    val syncDeleted: Boolean = false,
)
