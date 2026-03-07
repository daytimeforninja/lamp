package com.lamp.mobile.core.model

import org.junit.Assert.*
import org.junit.Test
import java.time.LocalDate
import java.time.LocalDateTime

class TaskTest {

    @Test
    fun `new task has TODO state`() {
        val task = Task(title = "Test")
        assertEquals(TaskState.TODO, task.state)
        assertNull(task.completed)
    }

    @Test
    fun `complete sets state and timestamp`() {
        val task = Task(title = "Test").complete()
        assertEquals(TaskState.DONE, task.state)
        assertNotNull(task.completed)
    }

    @Test
    fun `cancel sets state and timestamp`() {
        val task = Task(title = "Test").cancel()
        assertEquals(TaskState.CANCELLED, task.state)
        assertNotNull(task.completed)
    }

    @Test
    fun `isDone for completed states`() {
        assertTrue(TaskState.DONE.isDone)
        assertTrue(TaskState.CANCELLED.isDone)
        assertFalse(TaskState.TODO.isDone)
        assertFalse(TaskState.NEXT.isDone)
        assertFalse(TaskState.WAITING.isDone)
        assertFalse(TaskState.SOMEDAY.isDone)
    }

    @Test
    fun `isActive is inverse of isDone`() {
        for (state in TaskState.entries) {
            assertEquals(!state.isDone, state.isActive)
        }
    }

    @Test
    fun `isToday for scheduled task`() {
        val today = LocalDate.of(2025, 3, 5)
        val task = Task(title = "Test", scheduled = today)
        assertTrue(task.isToday(today))
    }

    @Test
    fun `isToday for overdue task`() {
        val today = LocalDate.of(2025, 3, 5)
        val task = Task(title = "Test", scheduled = today.minusDays(3))
        assertTrue(task.isToday(today))
    }

    @Test
    fun `isToday for deadline within 7 days`() {
        val today = LocalDate.of(2025, 3, 5)
        val task = Task(title = "Test", deadline = today.plusDays(5))
        assertTrue(task.isToday(today))
    }

    @Test
    fun `isToday false for done task`() {
        val today = LocalDate.of(2025, 3, 5)
        val task = Task(title = "Test", state = TaskState.DONE, scheduled = today)
        assertFalse(task.isToday(today))
    }

    @Test
    fun `hasContext checks list`() {
        val task = Task(title = "Test", contexts = listOf("@home", "@phone"))
        assertTrue(task.hasContext("@home"))
        assertFalse(task.hasContext("@office"))
    }

    @Test
    fun `priority from org format`() {
        assertEquals(Priority.A, Priority.fromOrg("A"))
        assertEquals(Priority.A, Priority.fromOrg("#A"))
        assertEquals(Priority.A, Priority.fromOrg("[#A]"))
        assertEquals(Priority.B, Priority.fromOrg("B"))
        assertEquals(Priority.C, Priority.fromOrg("C"))
        assertNull(Priority.fromOrg("X"))
    }

    @Test
    fun `priority from ical priority`() {
        assertEquals(Priority.A, Priority.fromIcalPriority(1))
        assertEquals(Priority.B, Priority.fromIcalPriority(3))
        assertEquals(Priority.B, Priority.fromIcalPriority(5))
        assertEquals(Priority.C, Priority.fromIcalPriority(7))
        assertEquals(Priority.C, Priority.fromIcalPriority(9))
        assertNull(Priority.fromIcalPriority(0))
    }

    @Test
    fun `task state keyword roundtrip`() {
        for (state in TaskState.entries) {
            assertEquals(state, TaskState.fromKeyword(state.keyword))
        }
    }
}
