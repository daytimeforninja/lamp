package com.lamp.mobile.core.database.dao

import androidx.room.*
import com.lamp.mobile.core.database.entity.CalendarEventEntity
import kotlinx.coroutines.flow.Flow

@Dao
interface CalendarEventDao {
    @Query("SELECT * FROM calendar_events ORDER BY start ASC")
    fun observeAll(): Flow<List<CalendarEventEntity>>

    @Query("SELECT * FROM calendar_events WHERE start >= :from AND start < :to ORDER BY start ASC")
    fun observeInRange(from: String, to: String): Flow<List<CalendarEventEntity>>

    @Query("SELECT * FROM calendar_events WHERE id = :id")
    suspend fun getById(id: String): CalendarEventEntity?

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsert(entity: CalendarEventEntity)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsertAll(entities: List<CalendarEventEntity>)

    @Query("DELETE FROM calendar_events WHERE id = :id")
    suspend fun deleteById(id: String)

    @Query("DELETE FROM calendar_events WHERE calendarHref = :calendarHref")
    suspend fun deleteByCalendar(calendarHref: String)

    @Query("SELECT * FROM calendar_events WHERE syncHref = :href")
    suspend fun getByHref(href: String): CalendarEventEntity?

    @Query("SELECT * FROM calendar_events")
    suspend fun getAll(): List<CalendarEventEntity>
}
