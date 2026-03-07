package com.lamp.mobile.feature.someday

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

data class SomedayUiState(val tasks: List<Task> = emptyList())

sealed class SomedayIntent {
    data class Activate(val taskId: UUID) : SomedayIntent()
    data class ToggleDone(val taskId: UUID) : SomedayIntent()
}

@HiltViewModel
class SomedayViewModel @Inject constructor(
    private val taskRepo: TaskRepository,
) : MviViewModel<SomedayUiState, SomedayIntent, Nothing>(SomedayUiState()) {

    init {
        taskRepo.observeSomeday()
            .onEach { tasks -> updateState { copy(tasks = tasks) } }
            .launchIn(viewModelScope)
    }

    override suspend fun handleIntent(intent: SomedayIntent) {
        when (intent) {
            is SomedayIntent.Activate -> {
                val task = taskRepo.getById(intent.taskId) ?: return
                taskRepo.save(task.copy(state = TaskState.TODO))
            }
            is SomedayIntent.ToggleDone -> {
                val task = taskRepo.getById(intent.taskId) ?: return
                taskRepo.save(task.complete())
            }
        }
    }
}
