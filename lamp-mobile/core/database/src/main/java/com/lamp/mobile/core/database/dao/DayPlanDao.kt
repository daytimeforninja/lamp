package com.lamp.mobile.core.database.dao

import androidx.room.*
import com.lamp.mobile.core.database.entity.DayPlanEntity
import kotlinx.coroutines.flow.Flow

@Dao
interface DayPlanDao {
    @Query("SELECT * FROM day_plans WHERE date = :date")
    fun observeByDate(date: String): Flow<DayPlanEntity?>

    @Query("SELECT * FROM day_plans WHERE date = :date")
    suspend fun getByDate(date: String): DayPlanEntity?

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsert(entity: DayPlanEntity)

    @Query("DELETE FROM day_plans WHERE date = :date")
    suspend fun deleteByDate(date: String)
}
