package com.lamp.mobile.feature.domode

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

data class DoModeUiState(
    val dayPlan: DayPlan = DayPlan(),
    val confirmedTasks: List<Task> = emptyList(),
    val pickedMedia: List<ListItem> = emptyList(),
    val pickedShopping: List<ListItem> = emptyList(),
)

sealed class DoModeIntent {
    data class MarkDone(val taskId: UUID) : DoModeIntent()
    data class ToggleMediaDone(val itemId: UUID) : DoModeIntent()
    data class ToggleShoppingDone(val itemId: UUID) : DoModeIntent()
}

@HiltViewModel
class DoModeViewModel @Inject constructor(
    private val dayPlanRepo: DayPlanRepository,
    private val taskRepo: TaskRepository,
    private val listItemRepo: ListItemRepository,
) : MviViewModel<DoModeUiState, DoModeIntent, Nothing>(DoModeUiState()) {

    init {
        combine(
            dayPlanRepo.observeByDate(LocalDate.now()),
            taskRepo.observeAll(),
            listItemRepo.observeByKind(ListKind.MEDIA),
            listItemRepo.observeByKind(ListKind.SHOPPING),
        ) { plan, allTasks, allMedia, allShopping ->
            val dayPlan = plan ?: DayPlan(date = LocalDate.now())
            val confirmed = allTasks.filter { it.id in dayPlan.confirmedTaskIds }
            val media = allMedia.filter { it.id in dayPlan.pickedMediaIds }
            val shopping = allShopping.filter { it.id in dayPlan.pickedShoppingIds }
            updateState { copy(dayPlan = dayPlan, confirmedTasks = confirmed,
                pickedMedia = media, pickedShopping = shopping) }
        }.launchIn(viewModelScope)
    }

    override suspend fun handleIntent(intent: DoModeIntent) {
        when (intent) {
            is DoModeIntent.MarkDone -> {
                val task = taskRepo.getById(intent.taskId) ?: return
                taskRepo.save(task.complete())
                val plan = dayPlanRepo.getByDate(java.time.LocalDate.now()) ?: currentState.dayPlan
                dayPlanRepo.save(plan.completeTask(task.id, task.title, task.esc))
            }
            is DoModeIntent.ToggleMediaDone -> {
                val item = listItemRepo.getById(intent.itemId) ?: return
                listItemRepo.save(item.copy(done = !item.done))
            }
            is DoModeIntent.ToggleShoppingDone -> {
                val item = listItemRepo.getById(intent.itemId) ?: return
                listItemRepo.save(item.copy(done = !item.done))
            }
        }
    }
}
