package com.lamp.mobile.feature.calendar

import androidx.lifecycle.viewModelScope
import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.data.repository.CalendarEventRepository
import com.lamp.mobile.core.model.CalendarEvent
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import java.time.LocalDate
import java.time.LocalDateTime
import java.time.YearMonth
import javax.inject.Inject

data class CalendarUiState(
    val currentMonth: YearMonth = YearMonth.now(),
    val selectedDate: LocalDate = LocalDate.now(),
    val events: List<CalendarEvent> = emptyList(),
)

sealed class CalendarIntent {
    data object PrevMonth : CalendarIntent()
    data object NextMonth : CalendarIntent()
    data class SelectDay(val date: LocalDate) : CalendarIntent()
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
        }
    }
}
