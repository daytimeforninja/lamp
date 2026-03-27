package com.lamp.mobile.core.database.di

import android.content.Context
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

    @Provides
    @Singleton
    fun provideDatabase(@ApplicationContext context: Context): LampDatabase =
        LampDatabase.build(context)

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
