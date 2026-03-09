package com.lamp.mobile.feature.notes

import androidx.lifecycle.viewModelScope
import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.common.ui.LinkSearchResult
import com.lamp.mobile.core.data.repository.ContactRepository
import com.lamp.mobile.core.data.repository.NoteRepository
import com.lamp.mobile.core.data.repository.ProjectRepository
import com.lamp.mobile.core.data.repository.TaskRepository
import com.lamp.mobile.core.model.Contact
import com.lamp.mobile.core.model.LinkTarget
import com.lamp.mobile.core.model.Note
import com.lamp.mobile.core.model.Project
import com.lamp.mobile.core.model.Task
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import java.time.LocalDateTime
import java.util.UUID
import javax.inject.Inject

data class NotesUiState(
    val notes: List<Note> = emptyList(),
    val inputText: String = "",
    val editingNote: Note? = null,
    val linkSearchResults: List<LinkSearchResult> = emptyList(),
    val backlinks: List<Note> = emptyList(),
)

sealed class NotesIntent {
    data class InputChanged(val text: String) : NotesIntent()
    data object Submit : NotesIntent()
    data class SelectNote(val note: Note) : NotesIntent()
    data object ClearSelection : NotesIntent()
    data class UpdateBody(val noteId: UUID, val body: String) : NotesIntent()
    data class UpdateTags(val noteId: UUID, val tags: String) : NotesIntent()
    data class UpdateSource(val noteId: UUID, val source: String) : NotesIntent()
    data class Delete(val noteId: UUID) : NotesIntent()
    data class LinkSearchChanged(val query: String) : NotesIntent()
    data class AddLink(val noteId: UUID, val target: LinkTarget) : NotesIntent()
    data class RemoveLink(val noteId: UUID, val target: LinkTarget) : NotesIntent()
}

@HiltViewModel
class NotesViewModel @Inject constructor(
    private val noteRepo: NoteRepository,
    private val taskRepo: TaskRepository,
    private val projectRepo: ProjectRepository,
    private val contactRepo: ContactRepository,
) : MviViewModel<NotesUiState, NotesIntent, Nothing>(NotesUiState()) {

    private var cachedTasks: List<Task> = emptyList()
    private var cachedProjects: List<Project> = emptyList()
    private var cachedContacts: List<Contact> = emptyList()

    init {
        noteRepo.observeAll()
            .onEach { notes ->
                val editing = currentState.editingNote
                val updatedEditing = editing?.let { ed -> notes.find { it.id == ed.id } }
                val backlinks = if (updatedEditing != null) {
                    notes.filter { n -> n.links.any { it.targetId == updatedEditing.id } }
                } else emptyList()
                updateState {
                    copy(
                        notes = notes,
                        editingNote = updatedEditing,
                        backlinks = backlinks,
                    )
                }
            }
            .launchIn(viewModelScope)

        taskRepo.observeAll()
            .onEach { cachedTasks = it }
            .launchIn(viewModelScope)
        projectRepo.observeAll()
            .onEach { cachedProjects = it }
            .launchIn(viewModelScope)
        contactRepo.observeAll()
            .onEach { cachedContacts = it }
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
            is NotesIntent.SelectNote -> {
                val backlinks = currentState.notes.filter { n ->
                    n.links.any { it.targetId == intent.note.id }
                }
                updateState { copy(editingNote = intent.note, backlinks = backlinks) }
            }
            is NotesIntent.ClearSelection -> updateState { copy(editingNote = null, linkSearchResults = emptyList(), backlinks = emptyList()) }
            is NotesIntent.UpdateBody -> {
                val note = noteRepo.getById(intent.noteId) ?: return
                noteRepo.save(note.copy(body = intent.body, modified = LocalDateTime.now()))
            }
            is NotesIntent.UpdateTags -> {
                val note = noteRepo.getById(intent.noteId) ?: return
                val tags = intent.tags.split(",").map { it.trim() }.filter { it.isNotEmpty() }
                noteRepo.save(note.copy(tags = tags, modified = LocalDateTime.now()))
            }
            is NotesIntent.UpdateSource -> {
                val note = noteRepo.getById(intent.noteId) ?: return
                noteRepo.save(note.copy(source = intent.source.ifBlank { null }, modified = LocalDateTime.now()))
            }
            is NotesIntent.Delete -> {
                noteRepo.delete(intent.noteId)
                updateState { copy(editingNote = null) }
            }
            is NotesIntent.LinkSearchChanged -> {
                val query = intent.query.trim().lowercase()
                if (query.length < 2) {
                    updateState { copy(linkSearchResults = emptyList()) }
                    return
                }
                val results = mutableListOf<LinkSearchResult>()
                // Search notes
                currentState.notes.filter { it.title.lowercase().contains(query) && it.id != currentState.editingNote?.id }
                    .take(3)
                    .forEach { results.add(LinkSearchResult(it.title, "Note", LinkTarget.NoteLink(it.id))) }
                // Search tasks
                cachedTasks.filter { it.title.lowercase().contains(query) }
                    .take(3)
                    .forEach { results.add(LinkSearchResult(it.title, "Task", LinkTarget.TaskLink(it.id))) }
                // Search projects
                cachedProjects.filter { it.name.lowercase().contains(query) }
                    .take(3)
                    .forEach { results.add(LinkSearchResult(it.name, "Project", LinkTarget.ProjectLink(it.id))) }
                // Search contacts
                cachedContacts.filter { it.name.lowercase().contains(query) }
                    .take(3)
                    .forEach { results.add(LinkSearchResult(it.name, "Contact", LinkTarget.ContactLink(it.id))) }
                updateState { copy(linkSearchResults = results) }
            }
            is NotesIntent.AddLink -> {
                val note = noteRepo.getById(intent.noteId) ?: return
                if (note.links.none { it.targetId == intent.target.targetId }) {
                    noteRepo.save(note.copy(links = note.links + intent.target, modified = LocalDateTime.now()))
                }
            }
            is NotesIntent.RemoveLink -> {
                val note = noteRepo.getById(intent.noteId) ?: return
                noteRepo.save(note.copy(links = note.links.filter { it.targetId != intent.target.targetId }, modified = LocalDateTime.now()))
            }
        }
    }
}
