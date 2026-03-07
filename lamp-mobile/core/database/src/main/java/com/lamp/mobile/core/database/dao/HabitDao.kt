package com.lamp.mobile.core.database.dao

import androidx.room.*
import com.lamp.mobile.core.database.entity.HabitEntity
import kotlinx.coroutines.flow.Flow

@Dao
interface HabitDao {
    @Query("SELECT * FROM habits")
    fun observeAll(): Flow<List<HabitEntity>>

    @Query("SELECT * FROM habits WHERE taskId = :taskId")
    suspend fun getByTaskId(taskId: String): HabitEntity?

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsert(entity: HabitEntity)

    @Query("DELETE FROM habits WHERE taskId = :taskId")
    suspend fun deleteByTaskId(taskId: String)

    @Query("SELECT * FROM habits")
    suspend fun getAll(): List<HabitEntity>
}
