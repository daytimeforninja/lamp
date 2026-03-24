package com.lamp.mobile.core.database.di

import android.content.Context
import androidx.room.Room
import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase
import com.lamp.mobile.core.database.LampDatabase
import com.lamp.mobile.core.database.dao.*
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
object DatabaseModule {

    private val MIGRATION_5_6 = object : Migration(5, 6) {
        override fun migrate(db: SupportSQLiteDatabase) {
            db.execSQL("ALTER TABLE tasks ADD COLUMN dayplanDate TEXT")
        }
    }

    private val MIGRATION_6_7 = object : Migration(6, 7) {
        override fun migrate(db: SupportSQLiteDatabase) {
            // Add groups column, default to existing category value
            db.execSQL("ALTER TABLE contacts ADD COLUMN groups TEXT NOT NULL DEFAULT ''")
            db.execSQL("UPDATE contacts SET groups = category WHERE category IS NOT NULL AND category != ''")
        }
    }

    private val MIGRATION_7_8 = object : Migration(7, 8) {
        override fun migrate(db: SupportSQLiteDatabase) {
            db.execSQL("ALTER TABLE tasks ADD COLUMN clockEntries TEXT NOT NULL DEFAULT '[]'")
        }
    }

    private val MIGRATION_8_9 = object : Migration(8, 9) {
        override fun migrate(db: SupportSQLiteDatabase) {
            db.execSQL("ALTER TABLE tasks ADD COLUMN dayplanBudget INTEGER")
        }
    }

    private val MIGRATION_9_10 = object : Migration(9, 10) {
        override fun migrate(db: SupportSQLiteDatabase) {
            // Clear stale content hashes — hash function changed (added completed field).
            // Combined with null-hash-accepts-remote logic, this enables clean first sync.
            db.execSQL("UPDATE tasks SET syncHash = NULL")
        }
    }

    @Provides
    @Singleton
    fun provideDatabase(@ApplicationContext context: Context): LampDatabase =
        Room.databaseBuilder(context, LampDatabase::class.java, "lamp.db")
            .fallbackToDestructiveMigrationFrom(1, 2, 3, 4)
            .addMigrations(MIGRATION_5_6, MIGRATION_6_7, MIGRATION_7_8, MIGRATION_8_9, MIGRATION_9_10)
            .enableMultiInstanceInvalidation()
            .build()

    @Provides fun provideTaskDao(db: LampDatabase): TaskDao = db.taskDao()
    @Provides fun provideProjectDao(db: LampDatabase): ProjectDao = db.projectDao()
    @Provides fun provideHabitDao(db: LampDatabase): HabitDao = db.habitDao()
    @Provides fun provideDayPlanDao(db: LampDatabase): DayPlanDao = db.dayPlanDao()
    @Provides fun provideCalendarEventDao(db: LampDatabase): CalendarEventDao = db.calendarEventDao()
    @Provides fun provideNoteDao(db: LampDatabase): NoteDao = db.noteDao()
    @Provides fun provideContactDao(db: LampDatabase): ContactDao = db.contactDao()
    @Provides fun provideListItemDao(db: LampDatabase): ListItemDao = db.listItemDao()
    @Provides fun provideSyncMetadataDao(db: LampDatabase): SyncMetadataDao = db.syncMetadataDao()
    @Provides fun provideAccountDao(db: LampDatabase): AccountDao = db.accountDao()
}
