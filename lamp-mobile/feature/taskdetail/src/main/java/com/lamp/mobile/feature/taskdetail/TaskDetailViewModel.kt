package com.lamp.mobile.feature.taskdetail

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.data.repository.ContactRepository
import com.lamp.mobile.core.data.repository.ProjectRepository
import com.lamp.mobile.core.data.repository.TaskRepository
import com.lamp.mobile.core.model.*
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.launch
import java.time.LocalDate
import java.util.UUID
import javax.inject.Inject

data class TaskDetailUiState(
    val task: Task? = null,
    val allProjects: List<String> = emptyList(),
    val allContexts: Set<String> = emptySet(),
    val allContacts: List<String> = emptyList(),
    val noteInput: String = "",
    val isLoading: Boolean = true,
)

sealed class TaskDetailIntent {
    data class Load(val taskId: UUID) : TaskDetailIntent()
    data class UpdateTitle(val title: String) : TaskDetailIntent()
    data class SetState(val state: TaskState) : TaskDetailIntent()
    data class SetPriority(val priority: Priority?) : TaskDetailIntent()
    data class AddContext(val context: String) : TaskDetailIntent()
    data class RemoveContext(val context: String) : TaskDetailIntent()
    data class SetScheduled(val date: LocalDate?) : TaskDetailIntent()
    data class SetDeadline(val date: LocalDate?) : TaskDetailIntent()
    data class SetScheduledTime(val time: String?) : TaskDetailIntent()
    data class SetDeadlineTime(val time: String?) : TaskDetailIntent()
    data class SetEsc(val esc: Int?) : TaskDetailIntent()
    data class SetRecurrence(val recurrence: Recurrence?) : TaskDetailIntent()
    data class SetProject(val project: String?) : TaskDetailIntent()
    data class SetWaitingFor(val waitingFor: String?) : TaskDetailIntent()
    data class SetFollowUp(val date: LocalDate?) : TaskDetailIntent()
    data class NoteInputChanged(val text: String) : TaskDetailIntent()
    data object AppendNote : TaskDetailIntent()
    data object Delete : TaskDetailIntent()
}

sealed class TaskDetailEffect {
    data object Deleted : TaskDetailEffect()
}

@HiltViewModel
class TaskDetailViewModel @Inject constructor(
    savedStateHandle: SavedStateHandle,
    private val taskRepo: TaskRepository,
    private val projectRepo: ProjectRepository,
    private val contactRepo: ContactRepository,
) : MviViewModel<TaskDetailUiState, TaskDetailIntent, TaskDetailEffect>(TaskDetailUiState()) {

    init {
        val taskId = savedStateHandle.get<String>("taskId")
        if (taskId != null) {
            val uuid = try { UUID.fromString(taskId) } catch (_: Exception) { null }
            if (uuid != null) onIntent(TaskDetailIntent.Load(uuid))
        }

        projectRepo.observeAll()
            .onEach { projects -> updateState { copy(allProjects = projects.map { it.name }) } }
            .launchIn(viewModelScope)

        taskRepo.observeAll()
            .onEach { tasks ->
                val contexts = tasks.flatMap { it.contexts }.toSet()
                updateState { copy(allContexts = contexts) }
            }
            .launchIn(viewModelScope)

        contactRepo.observeAll()
            .onEach { contacts -> updateState { copy(allContacts = contacts.map { it.name }) } }
            .launchIn(viewModelScope)
    }

    override suspend fun handleIntent(intent: TaskDetailIntent) {
        when (intent) {
            is TaskDetailIntent.Load -> {
                val task = taskRepo.getById(intent.taskId)
                updateState { copy(task = task, isLoading = false) }
            }
            is TaskDetailIntent.UpdateTitle -> saveField { it.copy(title = intent.title) }
            is TaskDetailIntent.SetState -> saveField { it.copy(state = intent.state) }
            is TaskDetailIntent.SetPriority -> saveField { it.copy(priority = intent.priority) }
            is TaskDetailIntent.AddContext -> saveField { it.copy(contexts = it.contexts + intent.context) }
            is TaskDetailIntent.RemoveContext -> saveField { it.copy(contexts = it.contexts - intent.context) }
            is TaskDetailIntent.SetScheduled -> saveField { it.copy(scheduled = intent.date) }
            is TaskDetailIntent.SetDeadline -> saveField { it.copy(deadline = intent.date) }
            is TaskDetailIntent.SetScheduledTime -> saveField { it.copy(scheduledTime = intent.time) }
            is TaskDetailIntent.SetDeadlineTime -> saveField { it.copy(deadlineTime = intent.time) }
            is TaskDetailIntent.SetEsc -> saveField { it.copy(esc = intent.esc) }
            is TaskDetailIntent.SetRecurrence -> saveField { it.copy(recurrence = intent.recurrence) }
            is TaskDetailIntent.SetProject -> saveField { it.copy(project = intent.project) }
            is TaskDetailIntent.SetWaitingFor -> saveField { it.copy(waitingFor = intent.waitingFor) }
            is TaskDetailIntent.SetFollowUp -> saveField { it.copy(followUp = intent.date) }
            is TaskDetailIntent.NoteInputChanged -> updateState { copy(noteInput = intent.text) }
            is TaskDetailIntent.AppendNote -> {
                val text = currentState.noteInput.trim()
                if (text.isNotEmpty()) {
                    saveField { task ->
                        val existing = task.notes.let { if (it.isBlank()) "" else "$it\n" }
                        task.copy(notes = existing + text)
                    }
                    updateState { copy(noteInput = "") }
                }
            }
            is TaskDetailIntent.Delete -> {
                val task = currentState.task ?: return
                taskRepo.delete(task.id)
                sendEffect(TaskDetailEffect.Deleted)
            }
        }
    }

    private suspend fun saveField(transform: (Task) -> Task) {
        val task = currentState.task ?: return
        val updated = transform(task)
        taskRepo.save(updated)
        updateState { copy(task = updated) }
    }
}
