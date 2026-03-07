package com.lamp.mobile.core.model

import org.junit.Assert.*
import org.junit.Test
import java.time.LocalDate
import java.util.UUID

class DayPlanTest {

    @Test
    fun `default spoon budget is 50`() {
        assertEquals(50, DayPlan().spoonBudget)
    }

    @Test
    fun `remainingBudget subtracts spent`() {
        val plan = DayPlan(spoonBudget = 50, spentSpoons = 30)
        assertEquals(20, plan.remainingBudget())
    }

    @Test
    fun `remainingBudget does not go negative`() {
        val plan = DayPlan(spoonBudget = 50, spentSpoons = 60)
        assertEquals(0, plan.remainingBudget())
    }

    @Test
    fun `completeTask moves to completed and adds spoons`() {
        val taskId = UUID.randomUUID()
        val plan = DayPlan(confirmedTaskIds = listOf(taskId), spentSpoons = 10)
        val updated = plan.completeTask(taskId, "Test", 15)
        assertFalse(taskId in updated.confirmedTaskIds)
        assertEquals(1, updated.completedTasks.size)
        assertEquals(25, updated.spentSpoons)
    }

    @Test
    fun `uncompleteTask reverses completion`() {
        val taskId = UUID.randomUUID()
        val plan = DayPlan(
            completedTasks = listOf(CompletedTask(taskId, "Test", 15)),
            spentSpoons = 15,
        )
        val updated = plan.uncompleteTask(taskId)
        assertTrue(taskId in updated.confirmedTaskIds)
        assertEquals(0, updated.completedTasks.size)
        assertEquals(0, updated.spentSpoons)
    }

    @Test
    fun `isStale when date differs`() {
        val plan = DayPlan(date = LocalDate.of(2025, 1, 1))
        assertTrue(plan.isStale(LocalDate.of(2025, 1, 2)))
        assertFalse(plan.isStale(LocalDate.of(2025, 1, 1)))
    }
}
