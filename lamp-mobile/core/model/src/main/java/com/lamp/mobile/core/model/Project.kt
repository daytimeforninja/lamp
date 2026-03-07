package com.lamp.mobile.core.model

import java.util.UUID

data class Project(
    val id: UUID = UUID.randomUUID(),
    val name: String,
    val tasks: List<Task> = emptyList(),
    val purpose: String = "",
    val outcome: String = "",
    val brainstorm: String = "",
) {
    fun nextAction(): Task? =
        tasks.firstOrNull { it.state == TaskState.NEXT }
            ?: tasks.firstOrNull { it.state == TaskState.TODO }

    fun isStuck(): Boolean {
        val hasActive = tasks.any { it.state.isActive }
        return hasActive && nextAction() == null
    }

    fun completionRatio(): Pair<Int, Int> {
        val done = tasks.count { it.state.isDone }
        return done to tasks.size
    }
}
