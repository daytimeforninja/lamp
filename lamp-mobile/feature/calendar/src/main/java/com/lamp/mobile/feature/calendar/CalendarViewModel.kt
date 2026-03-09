package com.lamp.mobile.feature.calendar

import androidx.lifecycle.viewModelScope
import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.data.repository.CalendarEventRepository
import com.lamp.mobile.core.model.CalendarEvent
import com.lamp.mobile.core.model.EventStatus
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import java.time.LocalDate
import java.time.LocalDateTime
import java.time.LocalTime
import java.time.YearMonth
import java.util.UUID
import javax.inject.Inject

data class EventFormState(
    val id: UUID? = null,
    val title: String = "",
    val allDay: Boolean = false,
    val startDate: LocalDate = LocalDate.now(),
    val startTime: LocalTime = LocalTime.of(9, 0),
    val endDate: LocalDate = LocalDate.now(),
    val endTime: LocalTime = LocalTime.of(10, 0),
    val location: String = "",
    val description: String = "",
    val calendarHref: String = "",
    val validationError: String? = null,
)

data class CalendarUiState(
    val currentMonth: YearMonth = YearMonth.now(),
    val selectedDate: LocalDate = LocalDate.now(),
    val events: List<CalendarEvent> = emptyList(),
    val eventForm: EventFormState? = null,
)

sealed class CalendarIntent {
    data object PrevMonth : CalendarIntent()
    data object NextMonth : CalendarIntent()
    data class SelectDay(val date: LocalDate) : CalendarIntent()
    data object CreateEvent : CalendarIntent()
    data class EditEvent(val id: UUID) : CalendarIntent()
    data class SetEventTitle(val title: String) : CalendarIntent()
    data class SetEventAllDay(val allDay: Boolean) : CalendarIntent()
    data class SetEventStartDate(val date: LocalDate) : CalendarIntent()
    data class SetEventStartTime(val time: LocalTime) : CalendarIntent()
    data class SetEventEndDate(val date: LocalDate) : CalendarIntent()
    data class SetEventEndTime(val time: LocalTime) : CalendarIntent()
    data class SetEventLocation(val location: String) : CalendarIntent()
    data class SetEventDescription(val description: String) : CalendarIntent()
    data object SubmitEvent : CalendarIntent()
    data object CancelEventForm : CalendarIntent()
    data class DeleteEvent(val id: UUID) : CalendarIntent()
}

@HiltViewModel
class CalendarViewModel @Inject constructor(
    private val eventRepo: CalendarEventRepository,
) : MviViewModel<CalendarUiState, CalendarIntent, Nothing>(CalendarUiState()) {

    init {
        eventRepo.observeAll()
            .onEach { events -> updateState { copy(events = events) } }
            .launchIn(viewModelScope)
    }

    override suspend fun handleIntent(intent: CalendarIntent) {
        when (intent) {
            is CalendarIntent.PrevMonth -> updateState { copy(currentMonth = currentMonth.minusMonths(1)) }
            is CalendarIntent.NextMonth -> updateState { copy(currentMonth = currentMonth.plusMonths(1)) }
            is CalendarIntent.SelectDay -> updateState { copy(selectedDate = intent.date) }
            is CalendarIntent.CreateEvent -> {
                val date = currentState.selectedDate
                updateState {
                    copy(eventForm = EventFormState(
                        startDate = date,
                        endDate = date,
                    ))
                }
            }
            is CalendarIntent.EditEvent -> {
                val event = eventRepo.getById(intent.id) ?: return
                updateState {
                    copy(eventForm = EventFormState(
                        id = event.id,
                        title = event.title,
                        allDay = event.allDay,
                        startDate = event.start.toLocalDate(),
                        startTime = event.start.toLocalTime(),
                        endDate = event.end.toLocalDate(),
                        endTime = event.end.toLocalTime(),
                        location = event.location,
                        description = event.description,
                        calendarHref = event.calendarHref,
                    ))
                }
            }
            is CalendarIntent.SetEventTitle -> updateForm { it.copy(title = intent.title) }
            is CalendarIntent.SetEventAllDay -> updateForm { it.copy(allDay = intent.allDay) }
            is CalendarIntent.SetEventStartDate -> updateForm { it.copy(startDate = intent.date) }
            is CalendarIntent.SetEventStartTime -> updateForm { it.copy(startTime = intent.time) }
            is CalendarIntent.SetEventEndDate -> updateForm { it.copy(endDate = intent.date) }
            is CalendarIntent.SetEventEndTime -> updateForm { it.copy(endTime = intent.time) }
            is CalendarIntent.SetEventLocation -> updateForm { it.copy(location = intent.location) }
            is CalendarIntent.SetEventDescription -> updateForm { it.copy(description = intent.description) }
            is CalendarIntent.SubmitEvent -> {
                val form = currentState.eventForm ?: return
                if (form.title.isBlank()) {
                    updateForm { it.copy(validationError = "Title required") }
                    return
                }
                val start = if (form.allDay) form.startDate.atStartOfDay()
                    else LocalDateTime.of(form.startDate, form.startTime)
                val end = if (form.allDay) form.endDate.atTime(23, 59)
                    else LocalDateTime.of(form.endDate, form.endTime)
                if (!end.isAfter(start)) {
                    updateForm { it.copy(validationError = "End must be after start") }
                    return
                }
                val event = CalendarEvent(
                    id = form.id ?: UUID.randomUUID(),
                    title = form.title,
                    start = start,
                    end = end,
                    allDay = form.allDay,
                    location = form.location,
                    description = form.description,
                    calendarHref = form.calendarHref,
                )
                eventRepo.save(event)
                updateState { copy(eventForm = null) }
            }
            is CalendarIntent.CancelEventForm -> updateState { copy(eventForm = null) }
            is CalendarIntent.DeleteEvent -> {
                eventRepo.delete(intent.id)
                updateState { copy(eventForm = null) }
            }
        }
    }

    private fun updateForm(transform: (EventFormState) -> EventFormState) {
        val form = currentState.eventForm ?: return
        updateState { copy(eventForm = transform(form).copy(validationError = null)) }
    }
}
