package com.lamp.mobile.feature.domode

import androidx.lifecycle.viewModelScope
import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.data.repository.DayPlanRepository
import com.lamp.mobile.core.data.repository.TaskRepository
import com.lamp.mobile.core.model.DayPlan
import com.lamp.mobile.core.model.Task
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.launchIn
import java.time.LocalDate
import java.util.UUID
import javax.inject.Inject

data class DoModeUiState(
    val dayPlan: DayPlan = DayPlan(),
    val confirmedTasks: List<Task> = emptyList(),
)

sealed class DoModeIntent {
    data class MarkDone(val taskId: UUID) : DoModeIntent()
}

@HiltViewModel
class DoModeViewModel @Inject constructor(
    private val dayPlanRepo: DayPlanRepository,
    private val taskRepo: TaskRepository,
) : MviViewModel<DoModeUiState, DoModeIntent, Nothing>(DoModeUiState()) {

    init {
        combine(
            dayPlanRepo.observeByDate(LocalDate.now()),
            taskRepo.observeAll(),
        ) { plan, allTasks ->
            val dayPlan = plan ?: DayPlan(date = LocalDate.now())
            val confirmed = allTasks.filter { it.id in dayPlan.confirmedTaskIds }
            updateState { copy(dayPlan = dayPlan, confirmedTasks = confirmed) }
        }.launchIn(viewModelScope)
    }

    override suspend fun handleIntent(intent: DoModeIntent) {
        when (intent) {
            is DoModeIntent.MarkDone -> {
                val task = taskRepo.getById(intent.taskId) ?: return
                taskRepo.save(task.complete())
                val plan = currentState.dayPlan
                dayPlanRepo.save(plan.completeTask(task.id, task.title, task.esc))
            }
        }
    }
}
