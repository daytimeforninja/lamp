@file:OptIn(ExperimentalMaterial3Api::class)

package com.lamp.mobile.feature.conflicts

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.common.ui.SyncPullRefreshBox
import com.lamp.mobile.core.model.SyncConflict
import com.lamp.mobile.core.model.Task

@Composable
fun ConflictsScreen(
    viewModel: ConflictsViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()

    SyncPullRefreshBox(modifier = Modifier.fillMaxSize()) {
        Column(modifier = Modifier.fillMaxSize()) {
            TopAppBar(title = { Text("Sync Conflicts") })

            if (state.conflicts.isEmpty()) {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        "No conflicts",
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            } else {
                LazyColumn(
                    modifier = Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(16.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    items(state.conflicts, key = { it.id }) { conflict ->
                        when (conflict) {
                            is SyncConflict.StateMismatch -> StateMismatchCard(
                                conflict = conflict,
                                onAcceptLocal = { viewModel.onIntent(ConflictsIntent.AcceptLocal(conflict.id)) },
                                onAcceptRemote = { viewModel.onIntent(ConflictsIntent.AcceptRemote(conflict.id)) },
                                onDelete = { viewModel.onIntent(ConflictsIntent.DeleteConflict(conflict.id)) },
                            )
                            is SyncConflict.RemoteOnly -> RemoteOnlyCard(
                                conflict = conflict,
                                onImport = { viewModel.onIntent(ConflictsIntent.ImportTask(conflict.id)) },
                                onDelete = { viewModel.onIntent(ConflictsIntent.DeleteConflict(conflict.id)) },
                            )
                            is SyncConflict.LocalOnly -> LocalOnlyCard(
                                conflict = conflict,
                                onKeep = { viewModel.onIntent(ConflictsIntent.AcceptLocal(conflict.id)) },
                                onDelete = { viewModel.onIntent(ConflictsIntent.DeleteConflict(conflict.id)) },
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun StateMismatchCard(
    conflict: SyncConflict.StateMismatch,
    onAcceptLocal: () -> Unit,
    onAcceptRemote: () -> Unit,
    onDelete: () -> Unit,
) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text("State Mismatch", style = MaterialTheme.typography.titleSmall,
                color = MaterialTheme.colorScheme.error)

            TaskSummary("Local", conflict.localTask)
            TaskSummary("Remote", conflict.remoteTask)

            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                FilledTonalButton(onClick = onAcceptLocal) { Text("Keep Local") }
                FilledTonalButton(onClick = onAcceptRemote) { Text("Accept Remote") }
                OutlinedButton(onClick = onDelete) { Text("Dismiss") }
            }
        }
    }
}

@Composable
private fun RemoteOnlyCard(
    conflict: SyncConflict.RemoteOnly,
    onImport: () -> Unit,
    onDelete: () -> Unit,
) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text("Remote Only", style = MaterialTheme.typography.titleSmall,
                color = MaterialTheme.colorScheme.tertiary)
            TaskSummary("Remote", conflict.remoteTask)
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                FilledTonalButton(onClick = onImport) { Text("Import") }
                OutlinedButton(onClick = onDelete) { Text("Dismiss") }
            }
        }
    }
}

@Composable
private fun LocalOnlyCard(
    conflict: SyncConflict.LocalOnly,
    onKeep: () -> Unit,
    onDelete: () -> Unit,
) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text("Local Only", style = MaterialTheme.typography.titleSmall)
            TaskSummary("Local", conflict.localTask)
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                FilledTonalButton(onClick = onKeep) { Text("Keep") }
                OutlinedButton(onClick = onDelete) { Text("Delete") }
            }
        }
    }
}

@Composable
private fun TaskSummary(label: String, task: Task) {
    Column {
        Text("$label: ${task.title}", style = MaterialTheme.typography.bodyMedium)
        Text("State: ${task.state.keyword} | Priority: ${task.priority?.name ?: "none"}",
            style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}
