package com.lamp.mobile.feature.waiting

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

data class WaitingUiState(
    val tasks: List<Task> = emptyList(),
)

sealed class WaitingIntent {
    data class ToggleDone(val taskId: UUID) : WaitingIntent()
}

@HiltViewModel
class WaitingViewModel @Inject constructor(
    private val taskRepo: TaskRepository,
) : MviViewModel<WaitingUiState, WaitingIntent, Nothing>(WaitingUiState()) {

    init {
        taskRepo.observeWaiting()
            .onEach { tasks -> updateState { copy(tasks = tasks) } }
            .launchIn(viewModelScope)
    }

    override suspend fun handleIntent(intent: WaitingIntent) {
        when (intent) {
            is WaitingIntent.ToggleDone -> {
                val task = taskRepo.getById(intent.taskId) ?: return
                if (task.state.isDone) {
                    taskRepo.save(task.copy(state = TaskState.WAITING, completed = null))
                } else {
                    taskRepo.save(task.complete())
                }
            }
        }
    }
}
