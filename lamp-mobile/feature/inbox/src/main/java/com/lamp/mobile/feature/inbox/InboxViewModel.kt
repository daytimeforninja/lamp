package com.lamp.mobile.feature.inbox

import android.content.Context
import androidx.lifecycle.viewModelScope
import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.data.repository.ProjectRepository
import com.lamp.mobile.core.data.repository.TaskRepository
import com.lamp.mobile.core.model.Task
import com.lamp.mobile.core.model.TaskState
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import java.util.UUID
import javax.inject.Inject

data class InboxUiState(
    val tasks: List<Task> = emptyList(),
    val inputText: String = "",
    val searchQuery: String = "",
    val showEnhancedCapture: Boolean = false,
    val allContexts: Set<String> = emptySet(),
    val allProjects: List<String> = emptyList(),
)

sealed class InboxIntent {
    data class InputChanged(val text: String) : InboxIntent()
    data object Submit : InboxIntent()
    data class ToggleDone(val taskId: UUID) : InboxIntent()
    data class Delete(val taskId: UUID) : InboxIntent()
    data class SetState(val taskId: UUID, val state: TaskState) : InboxIntent()
    data class SearchQueryChanged(val query: String) : InboxIntent()
    data object ShowEnhancedCapture : InboxIntent()
    data object HideEnhancedCapture : InboxIntent()
    data class SubmitEnhanced(
        val title: String,
        val state: TaskState,
        val priority: com.lamp.mobile.core.model.Priority?,
        val contexts: List<String>,
        val project: String?,
        val scheduled: java.time.LocalDate?,
        val esc: Int?,
    ) : InboxIntent()
}

sealed class InboxEffect {
    data class Error(val message: String) : InboxEffect()
}

@HiltViewModel
class InboxViewModel @Inject constructor(
    private val taskRepo: TaskRepository,
    private val projectRepo: ProjectRepository,
    @ApplicationContext private val context: Context,
) : MviViewModel<InboxUiState, InboxIntent, InboxEffect>(InboxUiState()) {

    init {
        taskRepo.observeInbox()
            .onEach { tasks ->
                val contexts = tasks.flatMap { it.contexts }.toSet()
                updateState { copy(tasks = tasks, allContexts = contexts + loadSavedContexts()) }
            }
            .launchIn(viewModelScope)

        projectRepo.observeAll()
            .onEach { projects -> updateState { copy(allProjects = projects.map { it.name }) } }
            .launchIn(viewModelScope)
    }

    private fun loadSavedContexts(): Set<String> {
        val prefs = context.getSharedPreferences("lamp_settings", Context.MODE_PRIVATE)
        return prefs.getStringSet("available_contexts", emptySet()) ?: emptySet()
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
            is InboxIntent.SearchQueryChanged -> updateState { copy(searchQuery = intent.query) }
            is InboxIntent.ShowEnhancedCapture -> updateState { copy(showEnhancedCapture = true) }
            is InboxIntent.HideEnhancedCapture -> updateState { copy(showEnhancedCapture = false) }
            is InboxIntent.SubmitEnhanced -> {
                val task = Task(
                    title = intent.title,
                    state = intent.state,
                    priority = intent.priority,
                    contexts = intent.contexts,
                    project = intent.project,
                    scheduled = intent.scheduled,
                    esc = intent.esc,
                )
                taskRepo.save(task)
                updateState { copy(showEnhancedCapture = false) }
            }
        }
    }
}
