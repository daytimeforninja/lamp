package com.lamp.mobile.core.model

import java.time.LocalDate
import java.util.UUID

data class Contact(
    val id: UUID = UUID.randomUUID(),
    val name: String,
    val email: String? = null,
    val phone: String? = null,
    val website: String? = null,
    val signal: String? = null,
    val preferredMethod: String? = null,
    /** Contact groups / categories (from vCard CATEGORIES, comma-separated). */
    val groups: List<String> = listOf("Personal"),
    val lastContacted: LocalDate? = null,
    val syncHref: String? = null,
    val syncEtag: String? = null,
    val syncDirty: Boolean = false,
    val syncDeleted: Boolean = false,
)
