package com.lamp.mobile.core.model

enum class TaskState(val keyword: String) {
    TODO("TODO"),
    NEXT("NEXT"),
    WAITING("WAITING"),
    SOMEDAY("SOMEDAY"),
    DONE("DONE"),
    CANCELLED("CANCELLED");

    val isDone: Boolean get() = this == DONE || this == CANCELLED
    val isActive: Boolean get() = !isDone

    companion object {
        fun fromKeyword(s: String): TaskState? = entries.find { it.keyword == s }
    }
}
