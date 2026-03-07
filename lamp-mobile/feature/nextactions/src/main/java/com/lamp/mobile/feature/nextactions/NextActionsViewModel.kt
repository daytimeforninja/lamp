package com.lamp.mobile.feature.nextactions

import androidx.lifecycle.viewModelScope
import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.data.repository.TaskRepository
import com.lamp.mobile.core.model.Priority
import com.lamp.mobile.core.model.Task
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import java.util.UUID
import javax.inject.Inject

data class NextActionsUiState(
    val tasks: List<Task> = emptyList(),
    val selectedContexts: Set<String> = emptySet(),
    val allContexts: Set<String> = emptySet(),
)

sealed class NextActionsIntent {
    data class ToggleContext(val context: String) : NextActionsIntent()
    data class ToggleDone(val taskId: UUID) : NextActionsIntent()
}

@HiltViewModel
class NextActionsViewModel @Inject constructor(
    private val taskRepo: TaskRepository,
) : MviViewModel<NextActionsUiState, NextActionsIntent, Nothing>(NextActionsUiState()) {

    init {
        taskRepo.observeNextActions()
            .onEach { tasks ->
                val contexts = tasks.flatMap { it.contexts }.toSet()
                updateState { copy(tasks = tasks, allContexts = contexts) }
            }
            .launchIn(viewModelScope)
    }

    override suspend fun handleIntent(intent: NextActionsIntent) {
        when (intent) {
            is NextActionsIntent.ToggleContext -> {
                updateState {
                    val newContexts = if (intent.context in selectedContexts)
                        selectedContexts - intent.context
                    else
                        selectedContexts + intent.context
                    copy(selectedContexts = newContexts)
                }
            }
            is NextActionsIntent.ToggleDone -> {
                val task = taskRepo.getById(intent.taskId) ?: return
                if (task.state.isDone) {
                    taskRepo.save(task.copy(state = com.lamp.mobile.core.model.TaskState.NEXT, completed = null))
                } else {
                    taskRepo.save(task.complete())
                }
            }
        }
    }
}
