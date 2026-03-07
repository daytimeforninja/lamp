package com.lamp.mobile.feature.inbox

import androidx.lifecycle.viewModelScope
import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.data.repository.TaskRepository
import com.lamp.mobile.core.model.Task
import com.lamp.mobile.core.model.TaskState
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.launch
import java.util.UUID
import javax.inject.Inject

data class InboxUiState(
    val tasks: List<Task> = emptyList(),
    val inputText: String = "",
)

sealed class InboxIntent {
    data class InputChanged(val text: String) : InboxIntent()
    data object Submit : InboxIntent()
    data class ToggleDone(val taskId: UUID) : InboxIntent()
    data class Delete(val taskId: UUID) : InboxIntent()
    data class SetState(val taskId: UUID, val state: TaskState) : InboxIntent()
}

sealed class InboxEffect {
    data class Error(val message: String) : InboxEffect()
}

@HiltViewModel
class InboxViewModel @Inject constructor(
    private val taskRepo: TaskRepository,
) : MviViewModel<InboxUiState, InboxIntent, InboxEffect>(InboxUiState()) {

    init {
        taskRepo.observeInbox()
            .onEach { tasks -> updateState { copy(tasks = tasks) } }
            .launchIn(viewModelScope)
    }

    override suspend fun handleIntent(intent: InboxIntent) {
        when (intent) {
            is InboxIntent.InputChanged -> updateState { copy(inputText = intent.text) }
            is InboxIntent.Submit -> {
                val title = currentState.inputText.trim()
                if (title.isNotEmpty()) {
                    taskRepo.save(Task(title = title))
                    updateState { copy(inputText = "") }
                }
            }
            is InboxIntent.ToggleDone -> {
                val task = taskRepo.getById(intent.taskId) ?: return
                if (task.state.isDone) {
                    taskRepo.save(task.copy(state = TaskState.TODO, completed = null))
                } else {
                    taskRepo.save(task.complete())
                }
            }
            is InboxIntent.Delete -> taskRepo.delete(intent.taskId)
            is InboxIntent.SetState -> {
                val task = taskRepo.getById(intent.taskId) ?: return
                taskRepo.save(task.copy(state = intent.state))
            }
        }
    }
}
