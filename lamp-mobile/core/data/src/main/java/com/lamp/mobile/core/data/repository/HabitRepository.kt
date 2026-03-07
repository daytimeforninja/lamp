package com.lamp.mobile.core.data.repository

import com.lamp.mobile.core.model.Habit
import kotlinx.coroutines.flow.Flow
import java.util.UUID

interface HabitRepository {
    fun observeAll(): Flow<List<Habit>>
    suspend fun getByTaskId(taskId: UUID): Habit?
    suspend fun save(habit: Habit)
    suspend fun saveHabitOnly(habit: Habit)
    suspend fun delete(taskId: UUID)
}
