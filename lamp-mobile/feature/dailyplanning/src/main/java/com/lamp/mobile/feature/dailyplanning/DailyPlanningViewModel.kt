package com.lamp.mobile.feature.dailyplanning

import androidx.lifecycle.viewModelScope
import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.data.repository.DayPlanRepository
import com.lamp.mobile.core.data.repository.ListItemRepository
import com.lamp.mobile.core.data.repository.TaskRepository
import com.lamp.mobile.core.model.DayPlan
import com.lamp.mobile.core.model.ListItem
import com.lamp.mobile.core.model.ListKind
import com.lamp.mobile.core.model.Task
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.launchIn
import java.time.LocalDate
import java.util.UUID
import javax.inject.Inject

data class DailyPlanningUiState(
    val dayPlan: DayPlan = DayPlan(),
    val availableTasks: List<Task> = emptyList(),
    val allContexts: Set<String> = emptySet(),
    val rejectedIds: Set<UUID> = emptySet(),
    val mediaItems: List<ListItem> = emptyList(),
    val shoppingItems: List<ListItem> = emptyList(),
    val isStale: Boolean = false,
)

sealed class DailyPlanningIntent {
    data class SetBudget(val budget: Int) : DailyPlanningIntent()
    data class ToggleContext(val context: String) : DailyPlanningIntent()
    data class ConfirmTask(val taskId: UUID) : DailyPlanningIntent()
    data class UnconfirmTask(val taskId: UUID) : DailyPlanningIntent()
    data class RejectSuggestion(val taskId: UUID) : DailyPlanningIntent()
    data class PickMediaItem(val itemId: UUID) : DailyPlanningIntent()
    data class UnpickMediaItem(val itemId: UUID) : DailyPlanningIntent()
    data class PickShoppingItem(val itemId: UUID) : DailyPlanningIntent()
    data class UnpickShoppingItem(val itemId: UUID) : DailyPlanningIntent()
    data object StartNewPlan : DailyPlanningIntent()
}

@HiltViewModel
class DailyPlanningViewModel @Inject constructor(
    private val dayPlanRepo: DayPlanRepository,
    private val taskRepo: TaskRepository,
    private val listItemRepo: ListItemRepository,
) : MviViewModel<DailyPlanningUiState, DailyPlanningIntent, Nothing>(DailyPlanningUiState()) {

    init {
        combine(
            dayPlanRepo.observeByDate(LocalDate.now()),
            taskRepo.observeNextActions(),
            listItemRepo.observeByKind(ListKind.MEDIA),
            listItemRepo.observeByKind(ListKind.SHOPPING),
        ) { plan, tasks, media, shopping ->
            val today = LocalDate.now()
            val dayPlan = plan ?: DayPlan(date = today)
            val contexts = tasks.flatMap { it.contexts }.toSet()
            val isStale = dayPlan.isStale(today)
            updateState {
                copy(
                    dayPlan = dayPlan,
                    availableTasks = tasks,
                    allContexts = contexts,
                    mediaItems = media.filter { !it.done },
                    shoppingItems = shopping.filter { !it.done },
                    isStale = isStale,
                )
            }
        }.launchIn(viewModelScope)
    }

    private suspend fun freshPlan(): DayPlan {
        val today = LocalDate.now()
        return dayPlanRepo.getByDate(today) ?: DayPlan(date = today)
    }

    override suspend fun handleIntent(intent: DailyPlanningIntent) {
        when (intent) {
            is DailyPlanningIntent.SetBudget -> {
                dayPlanRepo.save(freshPlan().copy(spoonBudget = intent.budget))
            }
            is DailyPlanningIntent.ToggleContext -> {
                val plan = freshPlan()
                val contexts = if (intent.context in plan.activeContexts)
                    plan.activeContexts - intent.context
                else plan.activeContexts + intent.context
                dayPlanRepo.save(plan.copy(activeContexts = contexts))
            }
            is DailyPlanningIntent.ConfirmTask -> {
                val plan = freshPlan()
                dayPlanRepo.save(plan.copy(confirmedTaskIds = plan.confirmedTaskIds + intent.taskId))
            }
            is DailyPlanningIntent.UnconfirmTask -> {
                val plan = freshPlan()
                dayPlanRepo.save(plan.copy(confirmedTaskIds = plan.confirmedTaskIds - intent.taskId))
            }
            is DailyPlanningIntent.RejectSuggestion -> {
                updateState { copy(rejectedIds = rejectedIds + intent.taskId) }
            }
            is DailyPlanningIntent.PickMediaItem -> {
                val plan = freshPlan()
                dayPlanRepo.save(plan.copy(pickedMediaIds = plan.pickedMediaIds + intent.itemId))
            }
            is DailyPlanningIntent.UnpickMediaItem -> {
                val plan = freshPlan()
                dayPlanRepo.save(plan.copy(pickedMediaIds = plan.pickedMediaIds - intent.itemId))
            }
            is DailyPlanningIntent.PickShoppingItem -> {
                val plan = freshPlan()
                dayPlanRepo.save(plan.copy(pickedShoppingIds = plan.pickedShoppingIds + intent.itemId))
            }
            is DailyPlanningIntent.UnpickShoppingItem -> {
                val plan = freshPlan()
                dayPlanRepo.save(plan.copy(pickedShoppingIds = plan.pickedShoppingIds - intent.itemId))
            }
            is DailyPlanningIntent.StartNewPlan -> {
                dayPlanRepo.save(DayPlan(date = LocalDate.now()))
                updateState { copy(rejectedIds = emptySet(), isStale = false) }
            }
        }
    }
}
