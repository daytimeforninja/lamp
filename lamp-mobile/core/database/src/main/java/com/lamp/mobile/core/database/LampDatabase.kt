package com.lamp.mobile.core.database

import android.content.Context
import androidx.room.Database
import androidx.room.Room
import androidx.room.RoomDatabase
import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase
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
    version = 10,
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

    companion object {
        val MIGRATIONS: Array<Migration> = arrayOf(
            object : Migration(5, 6) {
                override fun migrate(db: SupportSQLiteDatabase) {
                    db.execSQL("ALTER TABLE tasks ADD COLUMN dayplanDate TEXT")
                }
            },
            object : Migration(6, 7) {
                override fun migrate(db: SupportSQLiteDatabase) {
                    db.execSQL("ALTER TABLE contacts ADD COLUMN groups TEXT NOT NULL DEFAULT ''")
                    db.execSQL("UPDATE contacts SET groups = category WHERE category IS NOT NULL AND category != ''")
                }
            },
            object : Migration(7, 8) {
                override fun migrate(db: SupportSQLiteDatabase) {
                    db.execSQL("ALTER TABLE tasks ADD COLUMN clockEntries TEXT NOT NULL DEFAULT '[]'")
                }
            },
            object : Migration(8, 9) {
                override fun migrate(db: SupportSQLiteDatabase) {
                    db.execSQL("ALTER TABLE tasks ADD COLUMN dayplanBudget INTEGER")
                }
            },
            object : Migration(9, 10) {
                override fun migrate(db: SupportSQLiteDatabase) {
                    // Clear stale content hashes — hash function changed (added completed field).
                    // Combined with null-hash-accepts-remote logic, this enables clean first sync.
                    db.execSQL("UPDATE tasks SET syncHash = NULL")
                }
            },
        )

        fun build(context: Context): LampDatabase =
            Room.databaseBuilder(context, LampDatabase::class.java, "lamp.db")
                .fallbackToDestructiveMigrationFrom(1, 2, 3, 4)
                .addMigrations(*MIGRATIONS)
                .enableMultiInstanceInvalidation()
                .build()
    }
}
