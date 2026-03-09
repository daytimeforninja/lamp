package com.lamp.mobile.feature.inbox

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.common.ui.EnhancedCaptureSheet
import com.lamp.mobile.core.common.ui.QuickCaptureBar
import com.lamp.mobile.core.common.ui.SyncPullRefreshBox
import com.lamp.mobile.core.common.ui.TaskRow

@OptIn(ExperimentalMaterial3Api::class, ExperimentalFoundationApi::class)
@Composable
fun InboxScreen(
    onNavigateToTaskDetail: (java.util.UUID) -> Unit = {},
    viewModel: InboxViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()
    val filteredTasks = if (state.searchQuery.isBlank()) state.tasks
    else state.tasks.filter { task ->
        task.title.contains(state.searchQuery, ignoreCase = true) ||
            task.contexts.any { it.contains(state.searchQuery, ignoreCase = true) }
    }

    // Enhanced capture sheet
    if (state.showEnhancedCapture) {
        EnhancedCaptureSheet(
            onDismiss = { viewModel.onIntent(InboxIntent.HideEnhancedCapture) },
            onSubmit = { data ->
                viewModel.onIntent(InboxIntent.SubmitEnhanced(
                    title = data.title,
                    state = data.state,
                    priority = data.priority,
                    contexts = data.contexts,
                    project = data.project,
                    scheduled = data.scheduled,
                    esc = data.esc,
                ))
            },
            allContexts = state.allContexts,
            allProjects = state.allProjects,
        )
    }

    SyncPullRefreshBox(modifier = Modifier.fillMaxSize()) {
        Column(modifier = Modifier.fillMaxSize()) {
            TopAppBar(title = { Text("Inbox") })

            // Search bar
            OutlinedTextField(
                value = state.searchQuery,
                onValueChange = { viewModel.onIntent(InboxIntent.SearchQueryChanged(it)) },
                placeholder = { Text("Search tasks...") },
                leadingIcon = { Icon(Icons.Filled.Search, "Search") },
                modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 4.dp),
                singleLine = true,
            )

            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Box(modifier = Modifier.weight(1f)) {
                    QuickCaptureBar(
                        value = state.inputText,
                        onValueChange = { viewModel.onIntent(InboxIntent.InputChanged(it)) },
                        onSubmit = { viewModel.onIntent(InboxIntent.Submit) },
                    )
                }
                IconButton(
                    onClick = { viewModel.onIntent(InboxIntent.ShowEnhancedCapture) },
                ) {
                    Icon(Icons.Filled.Add, "Enhanced capture")
                }
            }

            if (filteredTasks.isEmpty()) {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        if (state.searchQuery.isNotBlank()) "No matching tasks" else "Inbox zero!",
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            } else {
                LazyColumn(modifier = Modifier.fillMaxSize()) {
                    items(filteredTasks, key = { it.id }) { task ->
                        TaskRow(
                            task = task,
                            onToggleDone = { viewModel.onIntent(InboxIntent.ToggleDone(task.id)) },
                            onClick = { onNavigateToTaskDetail(task.id) },
                        )
                    }
                }
            }
        }
    }
}
