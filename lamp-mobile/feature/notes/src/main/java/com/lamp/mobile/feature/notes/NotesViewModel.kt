package com.lamp.mobile.feature.notes

import androidx.lifecycle.viewModelScope
import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.data.repository.NoteRepository
import com.lamp.mobile.core.model.Note
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import java.time.LocalDateTime
import java.util.UUID
import javax.inject.Inject

data class NotesUiState(
    val notes: List<Note> = emptyList(),
    val inputText: String = "",
    val editingNote: Note? = null,
)

sealed class NotesIntent {
    data class InputChanged(val text: String) : NotesIntent()
    data object Submit : NotesIntent()
    data class SelectNote(val note: Note) : NotesIntent()
    data object ClearSelection : NotesIntent()
    data class UpdateBody(val noteId: UUID, val body: String) : NotesIntent()
    data class UpdateTags(val noteId: UUID, val tags: String) : NotesIntent()
    data class Delete(val noteId: UUID) : NotesIntent()
}

@HiltViewModel
class NotesViewModel @Inject constructor(
    private val noteRepo: NoteRepository,
) : MviViewModel<NotesUiState, NotesIntent, Nothing>(NotesUiState()) {

    init {
        noteRepo.observeAll()
            .onEach { notes ->
                updateState {
                    copy(
                        notes = notes,
                        editingNote = editingNote?.let { ed -> notes.find { it.id == ed.id } },
                    )
                }
            }
            .launchIn(viewModelScope)
    }

    override suspend fun handleIntent(intent: NotesIntent) {
        when (intent) {
            is NotesIntent.InputChanged -> updateState { copy(inputText = intent.text) }
            is NotesIntent.Submit -> {
                val title = currentState.inputText.trim()
                if (title.isNotEmpty()) {
                    noteRepo.save(Note(title = title))
                    updateState { copy(inputText = "") }
                }
            }
            is NotesIntent.SelectNote -> updateState { copy(editingNote = intent.note) }
            is NotesIntent.ClearSelection -> updateState { copy(editingNote = null) }
            is NotesIntent.UpdateBody -> {
                val note = noteRepo.getById(intent.noteId) ?: return
                noteRepo.save(note.copy(body = intent.body, modified = LocalDateTime.now()))
            }
            is NotesIntent.UpdateTags -> {
                val note = noteRepo.getById(intent.noteId) ?: return
                val tags = intent.tags.split(",").map { it.trim() }.filter { it.isNotEmpty() }
                noteRepo.save(note.copy(tags = tags, modified = LocalDateTime.now()))
            }
            is NotesIntent.Delete -> {
                noteRepo.delete(intent.noteId)
                updateState { copy(editingNote = null) }
            }
        }
    }
}
