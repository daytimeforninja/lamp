package com.lamp.mobile.core.model

data class CalendarInfo(
    val href: String,
    val displayName: String,
    val supportsVtodo: Boolean,
    val supportsVevent: Boolean,
)

enum class CalendarPurpose {
    TASKS,
    EVENTS,
    NONE,
}
