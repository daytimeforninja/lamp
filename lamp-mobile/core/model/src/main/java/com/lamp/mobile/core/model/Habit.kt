package com.lamp.mobile.core.model

import java.time.LocalDate
import java.time.LocalDateTime

data class Habit(
    val task: Task,
    val completions: List<LocalDateTime> = emptyList(),
    val streak: Int = 0,
    val bestStreak: Int = 0,
) {
    fun isDue(today: LocalDate): Boolean =
        completions.none { it.toLocalDate() == today }

    fun recalculateStreak(today: LocalDate): Habit {
        val uniqueDates = completions.map { it.toLocalDate() }.distinct().sorted()
        if (uniqueDates.isEmpty()) return copy(streak = 0)

        // Count consecutive days backwards from yesterday (today may not be done yet)
        var current = 0
        var checkDate = today.minusDays(1)
        while (uniqueDates.contains(checkDate)) {
            current++
            checkDate = checkDate.minusDays(1)
        }
        // Also count today if completed
        if (uniqueDates.contains(today)) {
            current++
        }

        // Best streak: sliding window over sorted dates
        var best = 1
        var run = 1
        for (i in 1 until uniqueDates.size) {
            if (uniqueDates[i] == uniqueDates[i - 1].plusDays(1)) {
                run++
                if (run > best) best = run
            } else {
                run = 1
            }
        }

        return copy(
            streak = current,
            bestStreak = maxOf(best, bestStreak, current),
        )
    }
}
