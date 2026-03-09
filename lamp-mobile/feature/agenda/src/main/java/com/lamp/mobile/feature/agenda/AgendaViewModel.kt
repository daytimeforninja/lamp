package com.lamp.mobile.feature.agenda

import androidx.lifecycle.viewModelScope
import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.data.repository.CalendarEventRepository
import com.lamp.mobile.core.data.repository.HabitRepository
import com.lamp.mobile.core.data.repository.TaskRepository
import com.lamp.mobile.core.model.CalendarEvent
import com.lamp.mobile.core.model.Habit
import com.lamp.mobile.core.model.Task
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.launchIn
import java.time.LocalDate
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter
import java.time.format.TextStyle
import java.util.Locale
import java.util.UUID
import javax.inject.Inject

data class AgendaDaySection(
    val date: LocalDate,
    val label: String,
    val events: List<CalendarEvent>,
    val scheduledTasks: List<Task>,
    val deadlineTasks: List<Task>,
    val habitsDue: List<Habit>,
)

data class AgendaUiState(
    val overdueTasks: List<Task> = emptyList(),
    val daySections: List<AgendaDaySection> = emptyList(),
)

sealed class AgendaIntent {
    data class ToggleDone(val taskId: UUID) : AgendaIntent()
    data class CompleteHabit(val taskId: UUID) : AgendaIntent()
}

@HiltViewModel
class AgendaViewModel @Inject constructor(
    private val taskRepo: TaskRepository,
    private val eventRepo: CalendarEventRepository,
    private val habitRepo: HabitRepository,
) : MviViewModel<AgendaUiState, AgendaIntent, Nothing>(AgendaUiState()) {

    init {
        combine(
            taskRepo.observeAll(),
            eventRepo.observeAll(),
            habitRepo.observeAll(),
        ) { allTasks, allEvents, habits ->
            val today = LocalDate.now()
            val horizon = today.plusDays(30)
            val activeTasks = allTasks.filter { it.state.isActive }
            val events = allEvents.filter {
                val d = it.start.toLocalDate()
                !d.isBefore(today) && !d.isAfter(horizon)
            }

            // Overdue tasks
            val overdue = activeTasks.filter { task ->
                val sched = task.scheduled
                val dead = task.deadline
                (sched != null && sched.isBefore(today)) ||
                    (dead != null && dead.isBefore(today))
            }

            // Build day sections
            val sections = (0L..30L).map { offset ->
                val date = today.plusDays(offset)
                val dayEvents = events.filter { it.start.toLocalDate() == date }
                    .sortedBy { it.start }
                val scheduled = activeTasks.filter { it.scheduled == date }
                val deadline = activeTasks.filter { it.deadline == date }
                val habitsDue = if (offset == 0L) habits.filter { it.isDue(today) } else emptyList()

                AgendaDaySection(
                    date = date,
                    label = formatDayLabel(date, today),
                    events = dayEvents,
                    scheduledTasks = scheduled,
                    deadlineTasks = deadline,
                    habitsDue = habitsDue,
                )
            }.filter { it.events.isNotEmpty() || it.scheduledTasks.isNotEmpty() ||
                    it.deadlineTasks.isNotEmpty() || it.habitsDue.isNotEmpty() }

            updateState { copy(overdueTasks = overdue, daySections = sections) }
        }.launchIn(viewModelScope)
    }

    override suspend fun handleIntent(intent: AgendaIntent) {
        when (intent) {
            is AgendaIntent.ToggleDone -> {
                val task = taskRepo.getById(intent.taskId) ?: return
                if (task.state.isDone) {
                    taskRepo.save(task.copy(state = com.lamp.mobile.core.model.TaskState.TODO, completed = null))
                } else {
                    taskRepo.save(task.complete())
                }
            }
            is AgendaIntent.CompleteHabit -> {
                val habit = habitRepo.getByTaskId(intent.taskId) ?: return
                val updated = habit.copy(
                    completions = habit.completions + LocalDateTime.now()
                ).recalculateStreak(LocalDate.now())
                habitRepo.save(updated)
            }
        }
    }

    private fun formatDayLabel(date: LocalDate, today: LocalDate): String {
        val dow = date.dayOfWeek.getDisplayName(TextStyle.FULL, Locale.getDefault())
        val dateStr = date.format(DateTimeFormatter.ofPattern("MMM d"))
        return when {
            date == today -> "Today, $dow, $dateStr"
            date == today.plusDays(1) -> "Tomorrow, $dow, $dateStr"
            else -> "$dow, $dateStr"
        }
    }
}
