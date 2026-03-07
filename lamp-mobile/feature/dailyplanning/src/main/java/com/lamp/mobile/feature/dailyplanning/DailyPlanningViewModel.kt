package com.lamp.mobile.feature.dailyplanning

import androidx.lifecycle.viewModelScope
import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.data.repository.DayPlanRepository
import com.lamp.mobile.core.data.repository.TaskRepository
import com.lamp.mobile.core.model.DayPlan
import com.lamp.mobile.core.model.Task
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.launch
import java.time.LocalDate
import java.util.UUID
import javax.inject.Inject

data class DailyPlanningUiState(
    val dayPlan: DayPlan = DayPlan(),
    val availableTasks: List<Task> = emptyList(),
    val allContexts: Set<String> = emptySet(),
)

sealed class DailyPlanningIntent {
    data class SetBudget(val budget: Int) : DailyPlanningIntent()
    data class ToggleContext(val context: String) : DailyPlanningIntent()
    data class ConfirmTask(val taskId: UUID) : DailyPlanningIntent()
    data class UnconfirmTask(val taskId: UUID) : DailyPlanningIntent()
}

@HiltViewModel
class DailyPlanningViewModel @Inject constructor(
    private val dayPlanRepo: DayPlanRepository,
    private val taskRepo: TaskRepository,
) : MviViewModel<DailyPlanningUiState, DailyPlanningIntent, Nothing>(DailyPlanningUiState()) {

    init {
        val today = LocalDate.now()
        combine(
            dayPlanRepo.observeByDate(today),
            taskRepo.observeNextActions(),
        ) { plan, tasks ->
            val dayPlan = plan ?: DayPlan(date = today)
            val contexts = tasks.flatMap { it.contexts }.toSet()
            updateState { copy(dayPlan = dayPlan, availableTasks = tasks, allContexts = contexts) }
        }.launchIn(viewModelScope)
    }

    override suspend fun handleIntent(intent: DailyPlanningIntent) {
        when (intent) {
            is DailyPlanningIntent.SetBudget -> {
                val updated = currentState.dayPlan.copy(spoonBudget = intent.budget)
                dayPlanRepo.save(updated)
            }
            is DailyPlanningIntent.ToggleContext -> {
                val plan = currentState.dayPlan
                val contexts = if (intent.context in plan.activeContexts)
                    plan.activeContexts - intent.context
                else plan.activeContexts + intent.context
                dayPlanRepo.save(plan.copy(activeContexts = contexts))
            }
            is DailyPlanningIntent.ConfirmTask -> {
                val plan = currentState.dayPlan
                dayPlanRepo.save(plan.copy(confirmedTaskIds = plan.confirmedTaskIds + intent.taskId))
            }
            is DailyPlanningIntent.UnconfirmTask -> {
                val plan = currentState.dayPlan
                dayPlanRepo.save(plan.copy(confirmedTaskIds = plan.confirmedTaskIds - intent.taskId))
            }
        }
    }
}
