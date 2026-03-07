package com.lamp.mobile.core.model

import java.time.LocalDate
import java.util.UUID

data class CompletedTask(
    val id: UUID,
    val title: String,
    val esc: Int?,
)

data class DayPlan(
    val date: LocalDate = LocalDate.now(),
    val spoonBudget: Int = 50,
    val activeContexts: List<String> = emptyList(),
    val confirmedTaskIds: List<UUID> = emptyList(),
    val completedTasks: List<CompletedTask> = emptyList(),
    val spentSpoons: Int = 0,
    val pickedMediaIds: List<UUID> = emptyList(),
    val pickedShoppingIds: List<UUID> = emptyList(),
) {
    fun isStale(today: LocalDate): Boolean = date != today

    fun remainingBudget(): Int = (spoonBudget - spentSpoons).coerceAtLeast(0)

    fun completeTask(taskId: UUID, title: String, esc: Int?): DayPlan = copy(
        confirmedTaskIds = confirmedTaskIds - taskId,
        completedTasks = completedTasks + CompletedTask(taskId, title, esc),
        spentSpoons = spentSpoons + (esc ?: 0),
    )

    fun uncompleteTask(taskId: UUID): DayPlan {
        val task = completedTasks.find { it.id == taskId } ?: return this
        return copy(
            confirmedTaskIds = confirmedTaskIds + taskId,
            completedTasks = completedTasks.filter { it.id != taskId },
            spentSpoons = (spentSpoons - (task.esc ?: 0)).coerceAtLeast(0),
        )
    }
}
