package com.lamp.mobile.core.model

import org.junit.Assert.*
import org.junit.Test
import java.time.LocalDate
import java.time.LocalDateTime

class HabitTest {

    @Test
    fun `isDue when no completions today`() {
        val habit = Habit(task = Task(title = "Test"))
        assertTrue(habit.isDue(LocalDate.now()))
    }

    @Test
    fun `not due when completed today`() {
        val now = LocalDateTime.now()
        val habit = Habit(task = Task(title = "Test"), completions = listOf(now))
        assertFalse(habit.isDue(now.toLocalDate()))
    }

    @Test
    fun `recalculate streak counts consecutive days`() {
        val today = LocalDate.of(2025, 3, 5)
        val completions = listOf(
            LocalDateTime.of(2025, 3, 3, 10, 0),
            LocalDateTime.of(2025, 3, 4, 10, 0),
            LocalDateTime.of(2025, 3, 5, 10, 0),
        )
        val habit = Habit(task = Task(title = "Test"), completions = completions)
        val updated = habit.recalculateStreak(today)
        assertEquals(3, updated.streak)
        assertEquals(3, updated.bestStreak)
    }

    @Test
    fun `streak breaks on gap`() {
        val today = LocalDate.of(2025, 3, 5)
        val completions = listOf(
            LocalDateTime.of(2025, 3, 1, 10, 0),
            // gap on March 2
            LocalDateTime.of(2025, 3, 3, 10, 0),
            LocalDateTime.of(2025, 3, 4, 10, 0),
            LocalDateTime.of(2025, 3, 5, 10, 0),
        )
        val habit = Habit(task = Task(title = "Test"), completions = completions)
        val updated = habit.recalculateStreak(today)
        assertEquals(3, updated.streak)
    }
}
