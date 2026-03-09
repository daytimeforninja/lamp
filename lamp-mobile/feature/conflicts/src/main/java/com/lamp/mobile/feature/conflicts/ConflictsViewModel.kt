package com.lamp.mobile.feature.conflicts

import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.data.repository.TaskRepository
import com.lamp.mobile.core.model.SyncConflict
import dagger.hilt.android.lifecycle.HiltViewModel
import java.util.UUID
import javax.inject.Inject

data class ConflictsUiState(
    val conflicts: List<SyncConflict> = emptyList(),
)

sealed class ConflictsIntent {
    data class AcceptLocal(val conflictId: UUID) : ConflictsIntent()
    data class AcceptRemote(val conflictId: UUID) : ConflictsIntent()
    data class ImportTask(val conflictId: UUID) : ConflictsIntent()
    data class DeleteConflict(val conflictId: UUID) : ConflictsIntent()
}

@HiltViewModel
class ConflictsViewModel @Inject constructor(
    private val taskRepo: TaskRepository,
) : MviViewModel<ConflictsUiState, ConflictsIntent, Nothing>(ConflictsUiState()) {

    override suspend fun handleIntent(intent: ConflictsIntent) {
        when (intent) {
            is ConflictsIntent.AcceptLocal -> {
                val conflict = currentState.conflicts.find { it.id == intent.conflictId } ?: return
                when (conflict) {
                    is SyncConflict.StateMismatch -> {
                        taskRepo.save(conflict.localTask)
                    }
                    is SyncConflict.LocalOnly -> {
                        taskRepo.save(conflict.localTask)
                    }
                    is SyncConflict.RemoteOnly -> { /* nothing to accept locally */ }
                }
                removeConflict(intent.conflictId)
            }
            is ConflictsIntent.AcceptRemote -> {
                val conflict = currentState.conflicts.find { it.id == intent.conflictId } ?: return
                when (conflict) {
                    is SyncConflict.StateMismatch -> {
                        taskRepo.save(conflict.remoteTask.copy(id = conflict.localTask.id), markDirty = false)
                    }
                    is SyncConflict.RemoteOnly -> {
                        taskRepo.save(conflict.remoteTask, markDirty = false)
                    }
                    is SyncConflict.LocalOnly -> { /* nothing to accept remotely */ }
                }
                removeConflict(intent.conflictId)
            }
            is ConflictsIntent.ImportTask -> {
                val conflict = currentState.conflicts.find { it.id == intent.conflictId } ?: return
                when (conflict) {
                    is SyncConflict.RemoteOnly -> taskRepo.save(conflict.remoteTask, markDirty = false)
                    else -> {}
                }
                removeConflict(intent.conflictId)
            }
            is ConflictsIntent.DeleteConflict -> removeConflict(intent.conflictId)
        }
    }

    fun addConflicts(newConflicts: List<SyncConflict>) {
        updateState { copy(conflicts = conflicts + newConflicts) }
    }

    private fun removeConflict(id: UUID) {
        updateState { copy(conflicts = conflicts.filter { it.id != id }) }
    }
}
