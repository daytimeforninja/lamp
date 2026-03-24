package com.lamp.mobile.core.database

import androidx.room.Database
import androidx.room.RoomDatabase
import com.lamp.mobile.core.database.dao.*
import com.lamp.mobile.core.database.entity.*

@Database(
    entities = [
        TaskEntity::class,
        ProjectEntity::class,
        HabitEntity::class,
        DayPlanEntity::class,
        CalendarEventEntity::class,
        NoteEntity::class,
        ContactEntity::class,
        ListItemEntity::class,
        SyncMetadataEntity::class,
        AccountEntity::class,
    ],
    version = 9,
    exportSchema = true,
)
abstract class LampDatabase : RoomDatabase() {
    abstract fun taskDao(): TaskDao
    abstract fun projectDao(): ProjectDao
    abstract fun habitDao(): HabitDao
    abstract fun dayPlanDao(): DayPlanDao
    abstract fun calendarEventDao(): CalendarEventDao
    abstract fun noteDao(): NoteDao
    abstract fun contactDao(): ContactDao
    abstract fun listItemDao(): ListItemDao
    abstract fun syncMetadataDao(): SyncMetadataDao
    abstract fun accountDao(): AccountDao
}
