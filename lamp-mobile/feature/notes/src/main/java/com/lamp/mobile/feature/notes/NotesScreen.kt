package com.lamp.mobile.feature.notes

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.common.ui.QuickCaptureBar
import java.time.format.DateTimeFormatter

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NotesScreen(
    viewModel: NotesViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()

    if (state.editingNote != null) {
        NoteEditorScreen(
            note = state.editingNote!!,
            onBack = { viewModel.onIntent(NotesIntent.ClearSelection) },
            onUpdateBody = { viewModel.onIntent(NotesIntent.UpdateBody(state.editingNote!!.id, it)) },
            onUpdateTags = { viewModel.onIntent(NotesIntent.UpdateTags(state.editingNote!!.id, it)) },
            onDelete = { viewModel.onIntent(NotesIntent.Delete(state.editingNote!!.id)) },
        )
        return
    }

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

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun NoteEditorScreen(
    note: com.lamp.mobile.core.model.Note,
    onBack: () -> Unit,
    onUpdateBody: (String) -> Unit,
    onUpdateTags: (String) -> Unit,
    onDelete: () -> Unit,
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
                value = note.body,
                onValueChange = onUpdateBody,
                label = { Text("Content") },
                modifier = Modifier.fillMaxWidth().weight(1f),
            )
        }
    }
}
