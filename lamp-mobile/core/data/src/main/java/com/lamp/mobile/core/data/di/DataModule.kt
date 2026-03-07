package com.lamp.mobile.core.data.di

import com.lamp.mobile.core.data.repository.*
import dagger.Binds
import dagger.Module
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent

@Module
@InstallIn(SingletonComponent::class)
abstract class DataModule {

    @Binds abstract fun bindTaskRepository(impl: TaskRepositoryImpl): TaskRepository
    @Binds abstract fun bindProjectRepository(impl: ProjectRepositoryImpl): ProjectRepository
    @Binds abstract fun bindHabitRepository(impl: HabitRepositoryImpl): HabitRepository
    @Binds abstract fun bindDayPlanRepository(impl: DayPlanRepositoryImpl): DayPlanRepository
    @Binds abstract fun bindCalendarEventRepository(impl: CalendarEventRepositoryImpl): CalendarEventRepository
    @Binds abstract fun bindNoteRepository(impl: NoteRepositoryImpl): NoteRepository
    @Binds abstract fun bindContactRepository(impl: ContactRepositoryImpl): ContactRepository
    @Binds abstract fun bindListItemRepository(impl: ListItemRepositoryImpl): ListItemRepository
    @Binds abstract fun bindSyncMetadataRepository(impl: SyncMetadataRepositoryImpl): SyncMetadataRepository
}
