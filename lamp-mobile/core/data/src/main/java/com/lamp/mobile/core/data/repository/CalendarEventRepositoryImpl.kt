package com.lamp.mobile.core.data.repository

import com.lamp.mobile.core.database.converter.EntityMappers.toDomain
import com.lamp.mobile.core.database.converter.EntityMappers.toEntity
import com.lamp.mobile.core.database.dao.CalendarEventDao
import com.lamp.mobile.core.model.CalendarEvent
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter
import java.util.UUID
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class CalendarEventRepositoryImpl @Inject constructor(
    private val calendarEventDao: CalendarEventDao,
) : CalendarEventRepository {

    override fun observeAll(): Flow<List<CalendarEvent>> =
        calendarEventDao.observeAll().map { list -> list.map { it.toDomain() } }

    override fun observeInRange(from: LocalDateTime, to: LocalDateTime): Flow<List<CalendarEvent>> =
        calendarEventDao.observeInRange(
            from.format(DateTimeFormatter.ISO_LOCAL_DATE_TIME),
            to.format(DateTimeFormatter.ISO_LOCAL_DATE_TIME),
        ).map { list -> list.map { it.toDomain() } }

    override suspend fun getById(id: UUID): CalendarEvent? =
        calendarEventDao.getById(id.toString())?.toDomain()

    override suspend fun save(event: CalendarEvent) =
        calendarEventDao.upsert(event.toEntity())

    override suspend fun saveAll(events: List<CalendarEvent>) =
        calendarEventDao.upsertAll(events.map { it.toEntity() })

    override suspend fun delete(id: UUID) =
        calendarEventDao.deleteById(id.toString())

    override suspend fun deleteByCalendar(calendarHref: String) =
        calendarEventDao.deleteByCalendar(calendarHref)

    override suspend fun getByHref(href: String): CalendarEvent? =
        calendarEventDao.getByHref(href)?.toDomain()
}
