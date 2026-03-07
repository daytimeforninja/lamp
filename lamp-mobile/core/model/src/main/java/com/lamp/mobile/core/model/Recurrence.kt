package com.lamp.mobile.core.model

import java.time.LocalDate

enum class RecurrenceUnit(val suffix: String) {
    DAY("d"),
    WEEK("w"),
    MONTH("m"),
    YEAR("y");

    companion object {
        fun fromSuffix(s: String): RecurrenceUnit? = entries.find { it.suffix == s.lowercase() }
    }
}

data class RecurrenceInterval(
    val count: Int,
    val unit: RecurrenceUnit,
) {
    fun addTo(date: LocalDate): LocalDate = when (unit) {
        RecurrenceUnit.DAY -> date.plusDays(count.toLong())
        RecurrenceUnit.WEEK -> date.plusWeeks(count.toLong())
        RecurrenceUnit.MONTH -> date.plusMonths(count.toLong())
        RecurrenceUnit.YEAR -> date.plusYears(count.toLong())
    }
}

sealed class Recurrence {
    abstract val interval: RecurrenceInterval

    /** Standard (+Nd): next from original date */
    data class Standard(override val interval: RecurrenceInterval) : Recurrence()

    /** Relative (.+Nd): next from completion date */
    data class Relative(override val interval: RecurrenceInterval) : Recurrence()

    /** Strict (++Nd): skip to next future date */
    data class Strict(override val interval: RecurrenceInterval) : Recurrence()

    fun nextDate(original: LocalDate, completed: LocalDate, today: LocalDate): LocalDate =
        when (this) {
            is Standard -> interval.addTo(original)
            is Relative -> interval.addTo(completed)
            is Strict -> {
                var next = original
                while (!next.isAfter(today)) {
                    next = interval.addTo(next)
                }
                next
            }
        }

    override fun toString(): String {
        val prefix = when (this) {
            is Standard -> "+"
            is Relative -> ".+"
            is Strict -> "++"
        }
        return "$prefix${interval.count}${interval.unit.suffix}"
    }

    companion object {
        private val PATTERN = Regex("""^(\.\+|\+\+|\+)(\d+)([dwmy])$""")

        fun parse(s: String): Recurrence? {
            val match = PATTERN.matchEntire(s.trim()) ?: return null
            val (prefix, countStr, unitStr) = match.destructured
            val count = countStr.toIntOrNull() ?: return null
            if (count == 0) return null
            val unit = RecurrenceUnit.fromSuffix(unitStr) ?: return null
            val interval = RecurrenceInterval(count, unit)
            return when (prefix) {
                "+" -> Standard(interval)
                ".+" -> Relative(interval)
                "++" -> Strict(interval)
                else -> null
            }
        }
    }
}
