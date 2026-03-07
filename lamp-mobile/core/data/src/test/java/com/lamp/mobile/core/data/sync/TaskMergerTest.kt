package com.lamp.mobile.core.data.sync

import com.lamp.mobile.core.model.*
import org.junit.Assert.*
import org.junit.Test
import java.time.LocalDate

class TaskMergerTest {

    private val base = Task(
        title = "Base task",
        state = TaskState.TODO,
        priority = Priority.B,
        notes = "original notes",
    )

    @Test
    fun `no changes returns base`() {
        val result = TaskMerger.mergeTasks(base, base, base)
        assertEquals(base.title, result.title)
        assertEquals(base.state, result.state)
    }

    @Test
    fun `only local changed keeps local`() {
        val local = base.copy(title = "Local title")
        val result = TaskMerger.mergeTasks(local, base, base)
        assertEquals("Local title", result.title)
    }

    @Test
    fun `only remote changed takes remote`() {
        val remote = base.copy(title = "Remote title")
        val result = TaskMerger.mergeTasks(base, remote, base)
        assertEquals("Remote title", result.title)
    }

    @Test
    fun `both changed to same value keeps it`() {
        val local = base.copy(title = "Same")
        val remote = base.copy(title = "Same")
        val result = TaskMerger.mergeTasks(local, remote, base)
        assertEquals("Same", result.title)
    }

    @Test
    fun `both changed differently remote wins`() {
        val local = base.copy(title = "Local")
        val remote = base.copy(title = "Remote")
        val result = TaskMerger.mergeTasks(local, remote, base)
        assertEquals("Remote", result.title)
    }

    @Test
    fun `merge is field-level`() {
        val local = base.copy(title = "Local title", notes = "original notes")
        val remote = base.copy(title = "Base task", priority = Priority.A)
        val result = TaskMerger.mergeTasks(local, remote, base)
        assertEquals("Local title", result.title) // local change
        assertEquals(Priority.A, result.priority) // remote change
        assertEquals("original notes", result.notes) // no change
    }

    @Test
    fun `merge optional fields`() {
        val local = base.copy(scheduled = LocalDate.of(2025, 3, 5))
        val remote = base.copy(deadline = LocalDate.of(2025, 3, 10))
        val result = TaskMerger.mergeTasks(local, remote, base)
        assertEquals(LocalDate.of(2025, 3, 5), result.scheduled) // local
        assertEquals(LocalDate.of(2025, 3, 10), result.deadline) // remote
    }

    @Test
    fun `state merge remote wins on conflict`() {
        val local = base.copy(state = TaskState.NEXT)
        val remote = base.copy(state = TaskState.DONE)
        val result = TaskMerger.mergeTasks(local, remote, base)
        assertEquals(TaskState.DONE, result.state) // remote wins
    }
}
