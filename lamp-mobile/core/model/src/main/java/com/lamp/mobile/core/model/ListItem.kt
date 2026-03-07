package com.lamp.mobile.core.model

import java.time.LocalDateTime
import java.util.UUID

enum class ListKind {
    MEDIA,
    SHOPPING,
}

data class ListItem(
    val id: UUID = UUID.randomUUID(),
    val title: String,
    val notes: String = "",
    val created: LocalDateTime = LocalDateTime.now(),
    val done: Boolean = false,
    val kind: ListKind = ListKind.MEDIA,
)
