package com.lamp.mobile.core.data.repository

import com.lamp.mobile.core.model.CalendarEvent
import kotlinx.coroutines.flow.Flow
import java.time.LocalDateTime
import java.util.UUID

interface CalendarEventRepository {
    fun observeAll(): Flow<List<CalendarEvent>>
    fun observeInRange(from: LocalDateTime, to: LocalDateTime): Flow<List<CalendarEvent>>
    suspend fun getById(id: UUID): CalendarEvent?
    suspend fun save(event: CalendarEvent)
    suspend fun saveAll(events: List<CalendarEvent>)
    suspend fun delete(id: UUID)
    suspend fun deleteByCalendar(calendarHref: String)
    suspend fun getByHref(href: String): CalendarEvent?
}
