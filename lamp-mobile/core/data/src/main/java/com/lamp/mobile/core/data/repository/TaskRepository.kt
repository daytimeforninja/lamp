package com.lamp.mobile.core.data.repository

import com.lamp.mobile.core.model.Task
import com.lamp.mobile.core.model.TaskState
import kotlinx.coroutines.flow.Flow
import java.util.UUID

interface TaskRepository {
    fun observeAll(): Flow<List<Task>>
    fun observeNonHabit(): Flow<List<Task>>
    fun observeByState(state: TaskState): Flow<List<Task>>
    fun observeInbox(): Flow<List<Task>>
    fun observeNextActions(): Flow<List<Task>>
    fun observeWaiting(): Flow<List<Task>>
    fun observeSomeday(): Flow<List<Task>>
    fun observeArchived(): Flow<List<Task>>
    fun observeByProject(projectName: String): Flow<List<Task>>
    suspend fun getById(id: UUID): Task?
    suspend fun save(task: Task, markDirty: Boolean = true)
    suspend fun delete(id: UUID)
    suspend fun getDirty(): List<Task>
    suspend fun getDeleted(): List<Triple<String, String?, String?>> // id, syncHref, syncEtag
    suspend fun markSynced(id: UUID, hash: Long, etag: String, href: String)
    suspend fun getByHref(href: String): Task?
    suspend fun getAll(): List<Task>
}
