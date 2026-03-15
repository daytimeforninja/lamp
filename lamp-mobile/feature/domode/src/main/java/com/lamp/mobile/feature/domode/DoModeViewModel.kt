package com.lamp.mobile.feature.domode

import androidx.lifecycle.viewModelScope
import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.data.repository.DayPlanRepository
import com.lamp.mobile.core.data.repository.HabitRepository
import com.lamp.mobile.core.data.repository.ListItemRepository
import com.lamp.mobile.core.data.repository.TaskRepository
import com.lamp.mobile.core.model.DayPlan
import com.lamp.mobile.core.model.Habit
import com.lamp.mobile.core.model.ListItem
import com.lamp.mobile.core.model.ListKind
import com.lamp.mobile.core.model.Task
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.launch
import java.time.LocalDate
import java.time.LocalDateTime
import java.util.UUID
import javax.inject.Inject

data class DoModeUiState(
    val dayPlan: DayPlan = DayPlan(),
    val confirmedTasks: List<Task> = emptyList(),
    val dueHabits: List<Habit> = emptyList(),
    val pickedMedia: List<ListItem> = emptyList(),
    val pickedShopping: List<ListItem> = emptyList(),
    val activeTaskId: UUID? = null,
    val activeTimerStart: LocalDateTime? = null,
    val timerTick: Long = 0, // incremented to force recomposition
)

sealed class DoModeIntent {
    data class MarkDone(val taskId: UUID) : DoModeIntent()
    data class CompleteHabit(val taskId: UUID) : DoModeIntent()
    data class ToggleMediaDone(val itemId: UUID) : DoModeIntent()
    data class ToggleShoppingDone(val itemId: UUID) : DoModeIntent()
    data class ToggleWorkTimer(val taskId: UUID) : DoModeIntent()
}

@HiltViewModel
class DoModeViewModel @Inject constructor(
    private val dayPlanRepo: DayPlanRepository,
    private val taskRepo: TaskRepository,
    private val habitRepo: HabitRepository,
    private val listItemRepo: ListItemRepository,
) : MviViewModel<DoModeUiState, DoModeIntent, Nothing>(DoModeUiState()) {

    init {
        combine(
            dayPlanRepo.observeByDate(LocalDate.now()),
            taskRepo.observeAll(),
            habitRepo.observeAll(),
            listItemRepo.observeByKind(ListKind.MEDIA),
            listItemRepo.observeByKind(ListKind.SHOPPING),
        ) { plan, allTasks, allHabits, allMedia, allShopping ->
            val today = LocalDate.now()
            val dayPlan = plan ?: DayPlan(date = today)
            val confirmed = allTasks.filter { it.id in dayPlan.confirmedTaskIds && !it.state.isDone }
            val due = allHabits.filter { it.isDue(today) }
            val media = allMedia.filter { it.id in dayPlan.pickedMediaIds }
            val shopping = allShopping.filter { it.id in dayPlan.pickedShoppingIds }
            updateState { copy(dayPlan = dayPlan, confirmedTasks = confirmed,
                dueHabits = due, pickedMedia = media, pickedShopping = shopping) }
        }.launchIn(viewModelScope)

        // Timer tick — update every second while a timer is active
        viewModelScope.launch {
            while (true) {
                delay(1000)
                if (currentState.activeTaskId != null) {
                    updateState { copy(timerTick = timerTick + 1) }
                }
            }
        }
    }

    override suspend fun handleIntent(intent: DoModeIntent) {
        when (intent) {
            is DoModeIntent.MarkDone -> {
                // Stop timer if running for this or any task
                stopTimerIfRunning()

                val task = taskRepo.getById(intent.taskId) ?: return
                taskRepo.save(task.complete())
                val plan = dayPlanRepo.getByDate(java.time.LocalDate.now()) ?: currentState.dayPlan
                dayPlanRepo.save(plan.completeTask(task.id, task.title, task.esc))
            }
            is DoModeIntent.CompleteHabit -> {
                val habit = habitRepo.getByTaskId(intent.taskId) ?: return
                val now = java.time.LocalDateTime.now()
                val updated = habit.copy(
                    completions = habit.completions + now,
                ).recalculateStreak(java.time.LocalDate.now())
                habitRepo.saveHabitOnly(updated)
                val task = taskRepo.getById(intent.taskId)
                if (task != null) {
                    taskRepo.save(task.copy(logbookEntries = task.logbookEntries + now))
                }
            }
            is DoModeIntent.ToggleMediaDone -> {
                val item = listItemRepo.getById(intent.itemId) ?: return
                listItemRepo.save(item.copy(done = !item.done))
            }
            is DoModeIntent.ToggleShoppingDone -> {
                val item = listItemRepo.getById(intent.itemId) ?: return
                listItemRepo.save(item.copy(done = !item.done))
            }
            is DoModeIntent.ToggleWorkTimer -> {
                val now = LocalDateTime.now()
                val currentActive = currentState.activeTaskId
                val currentStart = currentState.activeTimerStart

                if (currentActive != null && currentStart != null) {
                    // Stop the current timer and log clock entry
                    val task = taskRepo.getById(currentActive)
                    if (task != null) {
                        taskRepo.save(task.copy(
                            clockEntries = task.clockEntries + (currentStart to now),
                        ))
                    }

                    if (currentActive == intent.taskId) {
                        // Same task — just stop
                        updateState { copy(activeTaskId = null, activeTimerStart = null) }
                    } else {
                        // Different task — start new timer
                        updateState { copy(activeTaskId = intent.taskId, activeTimerStart = now) }
                    }
                } else {
                    // No timer running — start one
                    updateState { copy(activeTaskId = intent.taskId, activeTimerStart = now) }
                }
            }
        }
    }

    private suspend fun stopTimerIfRunning() {
        val activeId = currentState.activeTaskId
        val start = currentState.activeTimerStart
        if (activeId != null && start != null) {
            val now = LocalDateTime.now()
            val task = taskRepo.getById(activeId)
            if (task != null) {
                taskRepo.save(task.copy(
                    clockEntries = task.clockEntries + (start to now),
                ))
            }
            updateState { copy(activeTaskId = null, activeTimerStart = null) }
        }
    }
}
