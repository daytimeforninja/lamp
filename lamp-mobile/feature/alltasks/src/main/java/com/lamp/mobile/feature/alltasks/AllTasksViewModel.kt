package com.lamp.mobile.feature.alltasks

import androidx.lifecycle.viewModelScope
import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.data.repository.TaskRepository
import com.lamp.mobile.core.model.Task
import com.lamp.mobile.core.model.TaskState
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import java.util.UUID
import javax.inject.Inject

enum class SortColumn { STATE, PRIORITY, TITLE, CONTEXT, ESC, SCHEDULED, DEADLINE }

data class AllTasksUiState(
    val tasks: List<Task> = emptyList(),
    val sortColumn: SortColumn = SortColumn.STATE,
    val sortAscending: Boolean = true,
    val searchQuery: String = "",
)

sealed class AllTasksIntent {
    data class SetSort(val column: SortColumn) : AllTasksIntent()
    data class SetSearchQuery(val query: String) : AllTasksIntent()
    data class ToggleDone(val taskId: UUID) : AllTasksIntent()
}

@HiltViewModel
class AllTasksViewModel @Inject constructor(
    private val taskRepo: TaskRepository,
) : MviViewModel<AllTasksUiState, AllTasksIntent, Nothing>(AllTasksUiState()) {

    init {
        taskRepo.observeNonHabit()
            .onEach { tasks -> updateState { copy(tasks = tasks) } }
            .launchIn(viewModelScope)
    }

    override suspend fun handleIntent(intent: AllTasksIntent) {
        when (intent) {
            is AllTasksIntent.SetSort -> {
                updateState {
                    if (sortColumn == intent.column) {
                        copy(sortAscending = !sortAscending)
                    } else {
                        copy(sortColumn = intent.column, sortAscending = true)
                    }
                }
            }
            is AllTasksIntent.SetSearchQuery -> updateState { copy(searchQuery = intent.query) }
            is AllTasksIntent.ToggleDone -> {
                val task = taskRepo.getById(intent.taskId) ?: return
                if (task.state.isDone) {
                    taskRepo.save(task.copy(state = TaskState.TODO, completed = null))
                } else {
                    taskRepo.save(task.complete())
                }
            }
        }
    }

    companion object {
        private val STATE_ORDER = mapOf(
            TaskState.NEXT to 0,
            TaskState.TODO to 1,
            TaskState.WAITING to 2,
            TaskState.SOMEDAY to 3,
            TaskState.DONE to 4,
            TaskState.CANCELLED to 5,
        )

        fun sortTasks(tasks: List<Task>, column: SortColumn, ascending: Boolean): List<Task> {
            val comparator: Comparator<Task> = when (column) {
                SortColumn.STATE -> compareBy { STATE_ORDER[it.state] ?: 99 }
                SortColumn.PRIORITY -> compareBy<Task> { it.priority?.ordinal ?: Int.MAX_VALUE }
                SortColumn.TITLE -> compareBy(String.CASE_INSENSITIVE_ORDER) { it.title }
                SortColumn.CONTEXT -> compareBy<Task> { it.contexts.firstOrNull() ?: "\uFFFF" }
                SortColumn.ESC -> compareBy<Task> { it.esc ?: Int.MAX_VALUE }
                SortColumn.SCHEDULED -> compareBy<Task> { it.scheduled ?: java.time.LocalDate.MAX }
                SortColumn.DEADLINE -> compareBy<Task> { it.deadline ?: java.time.LocalDate.MAX }
            }
            return if (ascending) tasks.sortedWith(comparator) else tasks.sortedWith(comparator.reversed())
        }
    }
}
