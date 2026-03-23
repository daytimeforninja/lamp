package com.lamp.mobile.core.data.repository

import com.lamp.mobile.core.database.converter.EntityMappers.toDomain
import com.lamp.mobile.core.database.converter.EntityMappers.toEntity
import com.lamp.mobile.core.database.dao.HabitDao
import com.lamp.mobile.core.database.dao.TaskDao
import com.lamp.mobile.core.model.Habit
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import java.util.UUID
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class HabitRepositoryImpl @Inject constructor(
    private val habitDao: HabitDao,
    private val taskDao: TaskDao,
) : HabitRepository {

    override fun observeAll(): Flow<List<Habit>> =
        combine(
            habitDao.observeAll(),
            taskDao.observeAll(),
        ) { habits, tasks ->
            val tasksById = tasks.associateBy { it.id }
            habits.mapNotNull { habit ->
                val task = tasksById[habit.taskId]?.toDomain() ?: return@mapNotNull null
                habit.toDomain(task)
            }
        }

    override suspend fun getByTaskId(taskId: UUID): Habit? {
        val habitEntity = habitDao.getByTaskId(taskId.toString()) ?: return null
        val taskEntity = taskDao.getById(taskId.toString()) ?: return null
        return habitEntity.toDomain(taskEntity.toDomain())
    }

    override suspend fun save(habit: Habit) {
        // Copy completions into task.logbookEntries so they get synced via X-LAMP-LOGBOOK
        val taskWithLogbook = habit.task.copy(logbookEntries = habit.completions)
        taskDao.upsert(taskWithLogbook.toEntity(syncDirty = true, location = "habits"))
        habitDao.upsert(habit.toEntity())
    }

    override suspend fun saveHabitOnly(habit: Habit) {
        // Also stamp completions into task so they sync via X-LAMP-LOGBOOK
        val taskWithLogbook = habit.task.copy(logbookEntries = habit.completions)
        taskDao.upsert(taskWithLogbook.toEntity(syncDirty = true, location = "habits"))
        habitDao.upsert(habit.toEntity())
    }

    override suspend fun delete(taskId: UUID) {
        habitDao.deleteByTaskId(taskId.toString())
        val existing = taskDao.getById(taskId.toString())
        if (existing?.syncHref != null) {
            taskDao.upsert(existing.copy(syncDeleted = true, syncDirty = true))
        } else {
            taskDao.deleteById(taskId.toString())
        }
    }
}
