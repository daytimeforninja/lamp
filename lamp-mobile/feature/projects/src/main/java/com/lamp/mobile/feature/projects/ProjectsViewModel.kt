package com.lamp.mobile.feature.projects

import androidx.lifecycle.viewModelScope
import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.data.repository.ProjectRepository
import com.lamp.mobile.core.model.Project
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import java.util.UUID
import javax.inject.Inject

data class ProjectsUiState(
    val projects: List<Project> = emptyList(),
    val inputText: String = "",
    val selectedProject: Project? = null,
)

sealed class ProjectsIntent {
    data class InputChanged(val text: String) : ProjectsIntent()
    data object Submit : ProjectsIntent()
    data class SelectProject(val project: Project) : ProjectsIntent()
    data object ClearSelection : ProjectsIntent()
    data class Delete(val projectId: UUID) : ProjectsIntent()
    data class UpdatePurpose(val projectId: UUID, val text: String) : ProjectsIntent()
    data class UpdateOutcome(val projectId: UUID, val text: String) : ProjectsIntent()
    data class UpdateBrainstorm(val projectId: UUID, val text: String) : ProjectsIntent()
}

@HiltViewModel
class ProjectsViewModel @Inject constructor(
    private val projectRepo: ProjectRepository,
) : MviViewModel<ProjectsUiState, ProjectsIntent, Nothing>(ProjectsUiState()) {

    init {
        projectRepo.observeAll()
            .onEach { projects ->
                updateState {
                    copy(
                        projects = projects,
                        selectedProject = selectedProject?.let { sel ->
                            projects.find { it.id == sel.id }
                        },
                    )
                }
            }
            .launchIn(viewModelScope)
    }

    override suspend fun handleIntent(intent: ProjectsIntent) {
        when (intent) {
            is ProjectsIntent.InputChanged -> updateState { copy(inputText = intent.text) }
            is ProjectsIntent.Submit -> {
                val name = currentState.inputText.trim()
                if (name.isNotEmpty()) {
                    projectRepo.save(Project(name = name))
                    updateState { copy(inputText = "") }
                }
            }
            is ProjectsIntent.SelectProject -> updateState { copy(selectedProject = intent.project) }
            is ProjectsIntent.ClearSelection -> updateState { copy(selectedProject = null) }
            is ProjectsIntent.Delete -> {
                projectRepo.delete(intent.projectId)
                updateState { copy(selectedProject = null) }
            }
            is ProjectsIntent.UpdatePurpose -> {
                val proj = projectRepo.getById(intent.projectId) ?: return
                projectRepo.save(proj.copy(purpose = intent.text))
            }
            is ProjectsIntent.UpdateOutcome -> {
                val proj = projectRepo.getById(intent.projectId) ?: return
                projectRepo.save(proj.copy(outcome = intent.text))
            }
            is ProjectsIntent.UpdateBrainstorm -> {
                val proj = projectRepo.getById(intent.projectId) ?: return
                projectRepo.save(proj.copy(brainstorm = intent.text))
            }
        }
    }
}
