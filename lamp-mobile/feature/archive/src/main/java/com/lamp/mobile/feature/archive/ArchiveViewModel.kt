package com.lamp.mobile.feature.archive

import androidx.lifecycle.viewModelScope
import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.data.repository.TaskRepository
import com.lamp.mobile.core.model.Task
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import javax.inject.Inject

enum class ArchiveSortColumn { COMPLETED, TITLE, STATE }

data class ArchiveUiState(
    val tasks: List<Task> = emptyList(),
    val sortColumn: ArchiveSortColumn = ArchiveSortColumn.COMPLETED,
    val sortAscending: Boolean = false,
    val searchQuery: String = "",
)

sealed class ArchiveIntent {
    data class SetSort(val column: ArchiveSortColumn) : ArchiveIntent()
    data class SetSearchQuery(val query: String) : ArchiveIntent()
}

@HiltViewModel
class ArchiveViewModel @Inject constructor(
    private val taskRepo: TaskRepository,
) : MviViewModel<ArchiveUiState, ArchiveIntent, Nothing>(ArchiveUiState()) {

    init {
        taskRepo.observeArchived()
            .onEach { tasks -> updateState { copy(tasks = tasks) } }
            .launchIn(viewModelScope)
    }

    override suspend fun handleIntent(intent: ArchiveIntent) {
        when (intent) {
            is ArchiveIntent.SetSort -> {
                updateState {
                    if (sortColumn == intent.column) {
                        copy(sortAscending = !sortAscending)
                    } else {
                        copy(sortColumn = intent.column, sortAscending = true)
                    }
                }
            }
            is ArchiveIntent.SetSearchQuery -> updateState { copy(searchQuery = intent.query) }
        }
    }

    companion object {
        fun sortTasks(tasks: List<Task>, column: ArchiveSortColumn, ascending: Boolean): List<Task> {
            val comparator: Comparator<Task> = when (column) {
                ArchiveSortColumn.COMPLETED -> compareBy { it.completed ?: java.time.LocalDateTime.MIN }
                ArchiveSortColumn.TITLE -> compareBy(String.CASE_INSENSITIVE_ORDER) { it.title }
                ArchiveSortColumn.STATE -> compareBy { it.state.name }
            }
            return if (ascending) tasks.sortedWith(comparator) else tasks.sortedWith(comparator.reversed())
        }
    }
}
