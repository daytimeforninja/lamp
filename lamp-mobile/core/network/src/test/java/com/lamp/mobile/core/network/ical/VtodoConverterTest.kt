package com.lamp.mobile.core.network.ical

import com.lamp.mobile.core.model.*
import org.junit.Assert.*
import org.junit.Test
import java.time.LocalDate
import java.time.LocalDateTime
import java.util.UUID

class VtodoConverterTest {

    @Test
    fun `taskToVcalendar produces valid iCal`() {
        val task = Task(
            title = "Buy groceries",
            state = TaskState.NEXT,
            priority = Priority.B,
            contexts = listOf("@errands", "@phone"),
            scheduled = LocalDate.of(2025, 3, 5),
            deadline = LocalDate.of(2025, 3, 10),
            notes = "Get milk and eggs",
            esc = 15,
            project = "Household",
        )
        val ical = VtodoConverter.taskToVcalendar(task)
        assertTrue(ical.contains("BEGIN:VCALENDAR"))
        assertTrue(ical.contains("BEGIN:VTODO"))
        assertTrue(ical.contains("SUMMARY:Buy groceries"))
        assertTrue(ical.contains("STATUS:NEEDS-ACTION"))
        assertTrue(ical.contains("PRIORITY:5"))
        assertTrue(ical.contains("CATEGORIES:@errands,@phone"))
        assertTrue(ical.contains("DTSTART;VALUE=DATE:20250305"))
        assertTrue(ical.contains("DUE;VALUE=DATE:20250310"))
        assertTrue(ical.contains("DESCRIPTION:Get milk and eggs"))
        assertTrue(ical.contains("X-LAMP-STATE:NEXT"))
        assertTrue(ical.contains("X-LAMP-ESC:15"))
        assertTrue(ical.contains("X-LAMP-PROJECT:Household"))
        assertTrue(ical.contains("END:VTODO"))
        assertTrue(ical.contains("END:VCALENDAR"))
    }

    @Test
    fun `vcalendarToTask parses basic VTODO`() {
        val ical = """
            BEGIN:VCALENDAR
            BEGIN:VTODO
            UID:test-123
            SUMMARY:Test task
            STATUS:NEEDS-ACTION
            PRIORITY:1
            CATEGORIES:@home,@phone
            DTSTART;VALUE=DATE:20250305
            DUE;VALUE=DATE:20250310
            DESCRIPTION:Some notes
            X-LAMP-STATE:NEXT
            X-LAMP-ESC:25
            X-LAMP-PROJECT:MyProject
            X-LAMP-WAITING-FOR:Alice
            END:VTODO
            END:VCALENDAR
        """.trimIndent()

        val task = VtodoConverter.vcalendarToTask(ical)
        assertNotNull(task)
        assertEquals("Test task", task!!.title)
        assertEquals(TaskState.NEXT, task.state)
        assertEquals(Priority.A, task.priority)
        assertEquals(listOf("@home", "@phone"), task.contexts)
        assertEquals(LocalDate.of(2025, 3, 5), task.scheduled)
        assertEquals(LocalDate.of(2025, 3, 10), task.deadline)
        assertEquals("Some notes", task.notes)
        assertEquals(25, task.esc)
        assertEquals("MyProject", task.project)
        assertEquals("Alice", task.waitingFor)
        assertEquals("test-123", task.syncUid)
    }

    @Test
    fun `roundtrip preserves all fields`() {
        val original = Task(
            title = "Roundtrip test",
            state = TaskState.WAITING,
            priority = Priority.C,
            contexts = listOf("@office"),
            scheduled = LocalDate.of(2025, 6, 1),
            deadline = LocalDate.of(2025, 6, 15),
            notes = "Test notes",
            esc = 30,
            project = "Project X",
            waitingFor = "Bob",
            recurrence = Recurrence.parse("+1w"),
        )

        val ical = VtodoConverter.taskToVcalendar(original)
        val parsed = VtodoConverter.vcalendarToTask(ical)

        assertNotNull(parsed)
        assertEquals(original.title, parsed!!.title)
        assertEquals(original.state, parsed.state)
        assertEquals(original.priority, parsed.priority)
        assertEquals(original.contexts, parsed.contexts)
        assertEquals(original.scheduled, parsed.scheduled)
        assertEquals(original.deadline, parsed.deadline)
        assertEquals(original.notes, parsed.notes)
        assertEquals(original.esc, parsed.esc)
        assertEquals(original.project, parsed.project)
        assertEquals(original.waitingFor, parsed.waitingFor)
        assertEquals(original.recurrence, parsed.recurrence)
    }

    @Test
    fun `vcalendarToTask returns null for empty summary`() {
        val ical = """
            BEGIN:VCALENDAR
            BEGIN:VTODO
            UID:test
            SUMMARY:
            END:VTODO
            END:VCALENDAR
        """.trimIndent()
        assertNull(VtodoConverter.vcalendarToTask(ical))
    }

    @Test
    fun `status mapping without X-LAMP-STATE`() {
        val completed = """
            BEGIN:VCALENDAR
            BEGIN:VTODO
            UID:test
            SUMMARY:Done task
            STATUS:COMPLETED
            END:VTODO
            END:VCALENDAR
        """.trimIndent()
        assertEquals(TaskState.DONE, VtodoConverter.vcalendarToTask(completed)?.state)

        val cancelled = """
            BEGIN:VCALENDAR
            BEGIN:VTODO
            UID:test
            SUMMARY:Cancelled task
            STATUS:CANCELLED
            END:VTODO
            END:VCALENDAR
        """.trimIndent()
        assertEquals(TaskState.CANCELLED, VtodoConverter.vcalendarToTask(cancelled)?.state)
    }

    @Test
    fun `content hash is deterministic`() {
        val task = Task(
            title = "Hash test",
            state = TaskState.TODO,
            priority = Priority.A,
            contexts = listOf("@home"),
        )
        val hash1 = VtodoConverter.taskContentHash(task)
        val hash2 = VtodoConverter.taskContentHash(task)
        assertEquals(hash1, hash2)
    }

    @Test
    fun `content hash changes with field changes`() {
        val task = Task(title = "Hash test", state = TaskState.TODO)
        val hash1 = VtodoConverter.taskContentHash(task)
        val modified = task.copy(title = "Modified")
        val hash2 = VtodoConverter.taskContentHash(modified)
        assertNotEquals(hash1, hash2)
    }

    @Test
    fun `vtodoHref builds correct path`() {
        assertEquals(
            "/calendars/tasks/abc-123.ics",
            VtodoConverter.vtodoHref("/calendars/tasks/", "abc-123"),
        )
        assertEquals(
            "/calendars/tasks/abc-123.ics",
            VtodoConverter.vtodoHref("/calendars/tasks", "abc-123"),
        )
    }
}
