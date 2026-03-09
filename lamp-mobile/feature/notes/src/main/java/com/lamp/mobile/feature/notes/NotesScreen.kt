@file:OptIn(ExperimentalLayoutApi::class)

package com.lamp.mobile.feature.notes

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.common.ui.LinkSearchField
import com.lamp.mobile.core.common.ui.QuickCaptureBar
import com.lamp.mobile.core.common.ui.SyncPullRefreshBox
import com.lamp.mobile.core.model.LinkTarget
import java.time.format.DateTimeFormatter

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NotesScreen(
    viewModel: NotesViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()

    val editingNote = state.editingNote
    if (editingNote != null) {
        NoteEditorScreen(
            note = editingNote,
            linkSearchResults = state.linkSearchResults,
            backlinks = state.backlinks,
            onBack = { viewModel.onIntent(NotesIntent.ClearSelection) },
            onUpdateBody = { viewModel.onIntent(NotesIntent.UpdateBody(editingNote.id, it)) },
            onUpdateTags = { viewModel.onIntent(NotesIntent.UpdateTags(editingNote.id, it)) },
            onUpdateSource = { viewModel.onIntent(NotesIntent.UpdateSource(editingNote.id, it)) },
            onDelete = { viewModel.onIntent(NotesIntent.Delete(editingNote.id)) },
            onLinkSearchChanged = { viewModel.onIntent(NotesIntent.LinkSearchChanged(it)) },
            onAddLink = { viewModel.onIntent(NotesIntent.AddLink(editingNote.id, it)) },
            onRemoveLink = { viewModel.onIntent(NotesIntent.RemoveLink(editingNote.id, it)) },
            onNavigateToNote = { noteId ->
                val target = state.notes.find { it.id == noteId }
                if (target != null) viewModel.onIntent(NotesIntent.SelectNote(target))
            },
        )
        return
    }

    SyncPullRefreshBox(modifier = Modifier.fillMaxSize()) {
    Column(modifier = Modifier.fillMaxSize()) {
        TopAppBar(title = { Text("Notes") })

        QuickCaptureBar(
            value = state.inputText,
            onValueChange = { viewModel.onIntent(NotesIntent.InputChanged(it)) },
            onSubmit = { viewModel.onIntent(NotesIntent.Submit) },
            placeholder = "New note...",
        )

        LazyColumn(modifier = Modifier.fillMaxSize()) {
            items(state.notes, key = { it.id }) { note ->
                ListItem(
                    headlineContent = { Text(note.title) },
                    supportingContent = {
                        Text(
                            note.modified.format(DateTimeFormatter.ofPattern("MMM d, yyyy")),
                            style = MaterialTheme.typography.labelSmall,
                        )
                    },
                    trailingContent = if (note.tags.isNotEmpty()) {
                        { Text(note.tags.joinToString(", "), style = MaterialTheme.typography.labelSmall) }
                    } else null,
                    modifier = Modifier.clickable { viewModel.onIntent(NotesIntent.SelectNote(note)) },
                )
                HorizontalDivider()
            }
        }
    }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun NoteEditorScreen(
    note: com.lamp.mobile.core.model.Note,
    linkSearchResults: List<com.lamp.mobile.core.common.ui.LinkSearchResult>,
    backlinks: List<com.lamp.mobile.core.model.Note>,
    onBack: () -> Unit,
    onUpdateBody: (String) -> Unit,
    onUpdateTags: (String) -> Unit,
    onUpdateSource: (String) -> Unit,
    onDelete: () -> Unit,
    onLinkSearchChanged: (String) -> Unit,
    onAddLink: (LinkTarget) -> Unit,
    onRemoveLink: (LinkTarget) -> Unit,
    onNavigateToNote: (java.util.UUID) -> Unit,
) {
    Column(modifier = Modifier.fillMaxSize()) {
        TopAppBar(
            title = { Text(note.title) },
            navigationIcon = {
                IconButton(onClick = onBack) { Icon(Icons.Filled.ArrowBack, "Back") }
            },
            actions = {
                IconButton(onClick = onDelete) { Icon(Icons.Filled.Delete, "Delete") }
            },
        )

        Column(
            modifier = Modifier.fillMaxSize().padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            OutlinedTextField(
                value = note.tags.joinToString(", "),
                onValueChange = onUpdateTags,
                label = { Text("Tags (comma-separated)") },
                modifier = Modifier.fillMaxWidth(),
                singleLine = true,
            )

            OutlinedTextField(
                value = note.source ?: "",
                onValueChange = onUpdateSource,
                label = { Text("Source") },
                modifier = Modifier.fillMaxWidth(),
                singleLine = true,
            )

            // Links section
            if (note.links.isNotEmpty()) {
                Text("Links", style = MaterialTheme.typography.labelMedium)
                FlowRow(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                    note.links.forEach { link ->
                        InputChip(
                            selected = false,
                            onClick = { onRemoveLink(link) },
                            label = { Text(linkLabel(link)) },
                            trailingIcon = { Icon(Icons.Filled.Close, "Remove", modifier = Modifier.size(16.dp)) },
                        )
                    }
                }
            }

            // Add link search
            LinkSearchField(
                searchResults = linkSearchResults,
                onQueryChange = onLinkSearchChanged,
                onSelect = onAddLink,
                modifier = Modifier.fillMaxWidth(),
            )

            // Backlinks
            if (backlinks.isNotEmpty()) {
                Text("Backlinks", style = MaterialTheme.typography.labelMedium)
                backlinks.forEach { bl ->
                    TextButton(onClick = { onNavigateToNote(bl.id) }) {
                        Text(bl.title, style = MaterialTheme.typography.bodySmall)
                    }
                }
            }

            OutlinedTextField(
                value = note.body,
                onValueChange = onUpdateBody,
                label = { Text("Content") },
                modifier = Modifier.fillMaxWidth().weight(1f),
            )
        }
    }
}

private fun linkLabel(link: LinkTarget): String = when (link) {
    is LinkTarget.NoteLink -> "Note: ${link.targetId.toString().take(8)}"
    is LinkTarget.TaskLink -> "Task: ${link.targetId.toString().take(8)}"
    is LinkTarget.ProjectLink -> "Project: ${link.targetId.toString().take(8)}"
    is LinkTarget.ContactLink -> "Contact: ${link.targetId.toString().take(8)}"
    is LinkTarget.AccountLink -> "Account: ${link.targetId.toString().take(8)}"
    is LinkTarget.MediaItemLink -> "Media: ${link.targetId.toString().take(8)}"
    is LinkTarget.ShoppingItemLink -> "Shopping: ${link.targetId.toString().take(8)}"
}
