package com.lamp.mobile.core.database.dao

import androidx.room.*
import com.lamp.mobile.core.database.entity.TaskEntity
import kotlinx.coroutines.flow.Flow

@Dao
interface TaskDao {
    @Query("SELECT * FROM tasks WHERE syncDeleted = 0 ORDER BY created DESC")
    fun observeAll(): Flow<List<TaskEntity>>

    @Query("SELECT * FROM tasks WHERE syncDeleted = 0 AND state = :state ORDER BY created DESC")
    fun observeByState(state: String): Flow<List<TaskEntity>>

    @Query("SELECT * FROM tasks WHERE syncDeleted = 0 AND location = 'inbox' ORDER BY created DESC")
    fun observeInbox(): Flow<List<TaskEntity>>

    @Query("SELECT * FROM tasks WHERE syncDeleted = 0 AND state = 'NEXT' ORDER BY created DESC")
    fun observeNextActions(): Flow<List<TaskEntity>>

    @Query("SELECT * FROM tasks WHERE syncDeleted = 0 AND state = 'WAITING' ORDER BY created DESC")
    fun observeWaiting(): Flow<List<TaskEntity>>

    @Query("SELECT * FROM tasks WHERE syncDeleted = 0 AND state = 'SOMEDAY' ORDER BY created DESC")
    fun observeSomeday(): Flow<List<TaskEntity>>

    @Query("SELECT * FROM tasks WHERE syncDeleted = 0 AND project = :projectName ORDER BY created DESC")
    fun observeByProject(projectName: String): Flow<List<TaskEntity>>

    @Query("SELECT * FROM tasks WHERE id = :id")
    suspend fun getById(id: String): TaskEntity?

    @Query("SELECT * FROM tasks WHERE syncDirty = 1 AND syncDeleted = 0")
    suspend fun getDirty(): List<TaskEntity>

    @Query("SELECT * FROM tasks WHERE syncDeleted = 1")
    suspend fun getDeleted(): List<TaskEntity>

    @Query("SELECT * FROM tasks WHERE syncHref = :href")
    suspend fun getByHref(href: String): TaskEntity?

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsert(entity: TaskEntity)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsertAll(entities: List<TaskEntity>)

    @Query("DELETE FROM tasks WHERE id = :id")
    suspend fun deleteById(id: String)

    @Query("UPDATE tasks SET syncDirty = 0, syncHash = :hash, syncEtag = :etag, syncHref = :href WHERE id = :id")
    suspend fun markSynced(id: String, hash: Long, etag: String, href: String)

    @Query("SELECT * FROM tasks WHERE syncDeleted = 0")
    suspend fun getAll(): List<TaskEntity>
}
