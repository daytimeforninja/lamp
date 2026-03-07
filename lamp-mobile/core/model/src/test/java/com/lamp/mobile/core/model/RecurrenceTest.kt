package com.lamp.mobile.core.model

import org.junit.Assert.*
import org.junit.Test
import java.time.LocalDate

class RecurrenceTest {

    @Test
    fun `parse standard recurrence`() {
        val r = Recurrence.parse("+1w")
        assertTrue(r is Recurrence.Standard)
        assertEquals(1, r!!.interval.count)
        assertEquals(RecurrenceUnit.WEEK, r.interval.unit)
    }

    @Test
    fun `parse relative recurrence`() {
        val r = Recurrence.parse(".+2d")
        assertTrue(r is Recurrence.Relative)
        assertEquals(2, r!!.interval.count)
        assertEquals(RecurrenceUnit.DAY, r.interval.unit)
    }

    @Test
    fun `parse strict recurrence`() {
        val r = Recurrence.parse("++1m")
        assertTrue(r is Recurrence.Strict)
        assertEquals(1, r!!.interval.count)
        assertEquals(RecurrenceUnit.MONTH, r.interval.unit)
    }

    @Test
    fun `parse yearly recurrence`() {
        val r = Recurrence.parse("+1y")
        assertTrue(r is Recurrence.Standard)
        assertEquals(RecurrenceUnit.YEAR, r!!.interval.unit)
    }

    @Test
    fun `parse rejects zero count`() {
        assertNull(Recurrence.parse("+0d"))
    }

    @Test
    fun `parse rejects invalid format`() {
        assertNull(Recurrence.parse("invalid"))
        assertNull(Recurrence.parse(""))
        assertNull(Recurrence.parse("+d"))
        assertNull(Recurrence.parse("+1x"))
    }

    @Test
    fun `toString roundtrips`() {
        val inputs = listOf("+1d", "+2w", ".+3m", "++1y", "+10d")
        for (input in inputs) {
            assertEquals(input, Recurrence.parse(input)?.toString())
        }
    }

    @Test
    fun `standard nextDate adds from original`() {
        val r = Recurrence.parse("+1w")!!
        val original = LocalDate.of(2025, 1, 1)
        val completed = LocalDate.of(2025, 1, 10)
        val today = LocalDate.of(2025, 1, 10)
        assertEquals(LocalDate.of(2025, 1, 8), r.nextDate(original, completed, today))
    }

    @Test
    fun `relative nextDate adds from completed`() {
        val r = Recurrence.parse(".+3d")!!
        val original = LocalDate.of(2025, 1, 1)
        val completed = LocalDate.of(2025, 1, 10)
        val today = LocalDate.of(2025, 1, 10)
        assertEquals(LocalDate.of(2025, 1, 13), r.nextDate(original, completed, today))
    }

    @Test
    fun `strict nextDate advances past today`() {
        val r = Recurrence.parse("++1w")!!
        val original = LocalDate.of(2025, 1, 1)
        val completed = LocalDate.of(2025, 1, 20)
        val today = LocalDate.of(2025, 1, 20)
        // original + 1w = Jan 8, + 2w = Jan 15, + 3w = Jan 22 (first after today)
        assertEquals(LocalDate.of(2025, 1, 22), r.nextDate(original, completed, today))
    }

    @Test
    fun `month addition clamps day`() {
        val interval = RecurrenceInterval(1, RecurrenceUnit.MONTH)
        // Jan 31 + 1 month -> Feb 28 (non-leap)
        assertEquals(LocalDate.of(2025, 2, 28), interval.addTo(LocalDate.of(2025, 1, 31)))
        // Jan 31 + 1 month in leap year -> Feb 29
        assertEquals(LocalDate.of(2024, 2, 29), interval.addTo(LocalDate.of(2024, 1, 31)))
    }
}
