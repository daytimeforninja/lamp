package com.lamp.mobile.core.model

import java.time.LocalDate
import java.util.UUID

data class Account(
    val id: UUID = UUID.randomUUID(),
    val name: String,
    val url: String = "",
    val notes: String = "",
    val lastChecked: LocalDate? = null,
)
