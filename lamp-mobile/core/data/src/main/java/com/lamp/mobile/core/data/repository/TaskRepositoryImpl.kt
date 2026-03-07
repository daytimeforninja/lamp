package com.lamp.mobile.core.data.repository

import com.lamp.mobile.core.database.converter.EntityMappers.toDomain
import com.lamp.mobile.core.database.converter.EntityMappers.toEntity
import com.lamp.mobile.core.database.dao.TaskDao
import com.lamp.mobile.core.model.Task
import com.lamp.mobile.core.model.TaskState
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import java.util.UUID
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class TaskRepositoryImpl @Inject constructor(
    private val taskDao: TaskDao,
) : TaskRepository {

    override fun observeAll(): Flow<List<Task>> =
        taskDao.observeAll().map { list -> list.map { it.toDomain() } }

    override fun observeByState(state: TaskState): Flow<List<Task>> =
        taskDao.observeByState(state.keyword).map { list -> list.map { it.toDomain() } }

    override fun observeInbox(): Flow<List<Task>> =
        taskDao.observeInbox().map { list -> list.map { it.toDomain() } }

    override fun observeNextActions(): Flow<List<Task>> =
        taskDao.observeNextActions().map { list -> list.map { it.toDomain() } }

    override fun observeWaiting(): Flow<List<Task>> =
        taskDao.observeWaiting().map { list -> list.map { it.toDomain() } }

    override fun observeSomeday(): Flow<List<Task>> =
        taskDao.observeSomeday().map { list -> list.map { it.toDomain() } }

    override fun observeByProject(projectName: String): Flow<List<Task>> =
        taskDao.observeByProject(projectName).map { list -> list.map { it.toDomain() } }

    override suspend fun getById(id: UUID): Task? =
        taskDao.getById(id.toString())?.toDomain()

    override suspend fun save(task: Task, markDirty: Boolean) {
        val location = when (task.state) {
            TaskState.TODO -> "inbox"
            TaskState.NEXT -> "next"
            TaskState.WAITING -> "waiting"
            TaskState.SOMEDAY -> "someday"
            TaskState.DONE, TaskState.CANCELLED -> "archive"
        }
        taskDao.upsert(task.toEntity(syncDirty = markDirty, location = location))
    }

    override suspend fun delete(id: UUID) {
        val existing = taskDao.getById(id.toString())
        if (existing?.syncHref != null) {
            // Mark for remote deletion instead of local delete
            taskDao.upsert(existing.copy(syncDeleted = true, syncDirty = true))
        } else {
            taskDao.deleteById(id.toString())
        }
    }

    override suspend fun getDirty(): List<Task> =
        taskDao.getDirty().map { it.toDomain() }

    override suspend fun getDeleted(): List<Pair<String, String?>> =
        taskDao.getDeleted().map { it.id to it.syncHref }

    override suspend fun markSynced(id: UUID, hash: Long, etag: String, href: String) =
        taskDao.markSynced(id.toString(), hash, etag, href)

    override suspend fun getByHref(href: String): Task? =
        taskDao.getByHref(href)?.toDomain()

    override suspend fun getAll(): List<Task> =
        taskDao.getAll().map { it.toDomain() }
}
