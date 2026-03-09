package com.lamp.mobile.feature.review

import androidx.lifecycle.viewModelScope
import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.data.repository.*
import com.lamp.mobile.core.model.*
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.launchIn
import java.time.LocalDate
import java.time.LocalDateTime
import java.time.temporal.ChronoUnit
import javax.inject.Inject

data class WaitingTaskInfo(
    val task: Task,
    val ageDays: Long,
)

data class HabitStatInfo(
    val name: String,
    val thisWeek: Int,
    val streak: Int,
)

data class ReviewUiState(
    val completedSteps: Set<Int> = emptySet(),
    val inboxCount: Int = 0,
    val nextActionsCount: Int = 0,
    val waitingTasks: List<WaitingTaskInfo> = emptyList(),
    val stuckProjects: List<Project> = emptyList(),
    val somedayCount: Int = 0,
    val upcomingEvents: List<CalendarEvent> = emptyList(),
    val habitStats: List<HabitStatInfo> = emptyList(),
)

sealed class ReviewIntent {
    data class ToggleStep(val index: Int) : ReviewIntent()
}

@HiltViewModel
class ReviewViewModel @Inject constructor(
    taskRepo: TaskRepository,
    projectRepo: ProjectRepository,
    eventRepo: CalendarEventRepository,
    habitRepo: HabitRepository,
) : MviViewModel<ReviewUiState, ReviewIntent, Nothing>(ReviewUiState()) {

    init {
        combine(
            taskRepo.observeInbox(),
            taskRepo.observeNextActions(),
            taskRepo.observeWaiting(),
            taskRepo.observeSomeday(),
        ) { inbox, next, waiting, someday ->
            val today = LocalDate.now()
            val waitingInfos = waiting.map { task ->
                val age = task.created.toLocalDate().let { ChronoUnit.DAYS.between(it, today) }
                WaitingTaskInfo(task, age)
            }
            updateState {
                copy(
                    inboxCount = inbox.size,
                    nextActionsCount = next.size,
                    waitingTasks = waitingInfos,
                    somedayCount = someday.size,
                )
            }
        }.launchIn(viewModelScope)

        projectRepo.observeAll()
            .combine(eventRepo.observeAll()) { projects, events ->
                val today = LocalDate.now()
                val twoWeeks = today.plusDays(14)
                val upcoming = events.filter {
                    val d = it.start.toLocalDate()
                    !d.isBefore(today) && !d.isAfter(twoWeeks)
                }
                val stuck = projects.filter { it.isStuck() }
                updateState { copy(stuckProjects = stuck, upcomingEvents = upcoming) }
            }.launchIn(viewModelScope)

        habitRepo.observeAll()
            .combine(taskRepo.observeAll()) { habits, _ ->
                val today = LocalDate.now()
                val weekStart = today.minusDays(6)
                val stats = habits.map { habit ->
                    val thisWeekCount = habit.completions.count { c ->
                        val d = c.toLocalDate()
                        !d.isBefore(weekStart) && !d.isAfter(today)
                    }
                    HabitStatInfo(habit.task.title, thisWeekCount, habit.streak)
                }
                updateState { copy(habitStats = stats) }
            }.launchIn(viewModelScope)
    }

    override suspend fun handleIntent(intent: ReviewIntent) {
        when (intent) {
            is ReviewIntent.ToggleStep -> updateState {
                copy(completedSteps = if (intent.index in completedSteps)
                    completedSteps - intent.index else completedSteps + intent.index)
            }
        }
    }
}
