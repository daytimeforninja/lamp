package com.lamp.mobile.core.data.repository

import com.lamp.mobile.core.model.DayPlan
import kotlinx.coroutines.flow.Flow
import java.time.LocalDate

interface DayPlanRepository {
    fun observeByDate(date: LocalDate): Flow<DayPlan?>
    suspend fun getByDate(date: LocalDate): DayPlan?
    suspend fun save(dayPlan: DayPlan)
}
