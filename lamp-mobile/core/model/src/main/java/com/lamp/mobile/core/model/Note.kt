package com.lamp.mobile.core.model

import java.time.LocalDateTime
import java.util.UUID

data class Note(
    val id: UUID = UUID.randomUUID(),
    val title: String,
    val body: String = "",
    val tags: List<String> = emptyList(),
    val links: List<LinkTarget> = emptyList(),
    val source: String? = null,
    val created: LocalDateTime = LocalDateTime.now(),
    val modified: LocalDateTime = LocalDateTime.now(),
    val syncEtag: String? = null,
)
