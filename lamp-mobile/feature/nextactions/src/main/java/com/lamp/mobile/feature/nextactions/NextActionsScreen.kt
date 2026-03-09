package com.lamp.mobile.feature.nextactions

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.common.ui.ContextChipRow
import com.lamp.mobile.core.common.ui.SyncPullRefreshBox
import com.lamp.mobile.core.common.ui.TaskRow
import com.lamp.mobile.core.model.Priority

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NextActionsScreen(
    onNavigateToTaskDetail: (java.util.UUID) -> Unit = {},
    viewModel: NextActionsViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()

    var tasksAfterSearch = state.tasks
    if (state.searchQuery.isNotBlank()) {
        tasksAfterSearch = tasksAfterSearch.filter { task ->
            task.title.contains(state.searchQuery, ignoreCase = true) ||
                task.contexts.any { it.contains(state.searchQuery, ignoreCase = true) }
        }
    }
    val filteredTasks = if (state.selectedContexts.isEmpty()) tasksAfterSearch
    else tasksAfterSearch.filter { task -> task.contexts.any { it in state.selectedContexts } }

    val grouped = filteredTasks.groupBy { it.priority ?: Priority.C }
        .toSortedMap()

    SyncPullRefreshBox(modifier = Modifier.fillMaxSize()) {
        Column(modifier = Modifier.fillMaxSize()) {
            TopAppBar(title = { Text("Next Actions") })

            // Search bar
            OutlinedTextField(
                value = state.searchQuery,
                onValueChange = { viewModel.onIntent(NextActionsIntent.SearchQueryChanged(it)) },
                placeholder = { Text("Search tasks...") },
                leadingIcon = { Icon(Icons.Filled.Search, "Search") },
                modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 4.dp),
                singleLine = true,
            )

            if (state.allContexts.isNotEmpty()) {
                Row(
                    modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp),
                    horizontalArrangement = Arrangement.spacedBy(4.dp),
                ) {
                    state.allContexts.forEach { ctx ->
                        FilterChip(
                            selected = ctx in state.selectedContexts,
                            onClick = { viewModel.onIntent(NextActionsIntent.ToggleContext(ctx)) },
                            label = { Text(ctx) },
                        )
                    }
                }
            }

            LazyColumn(modifier = Modifier.fillMaxSize()) {
                grouped.forEach { (priority, tasks) ->
                    item {
                        Text(
                            text = "Priority ${priority.name}",
                            style = MaterialTheme.typography.titleSmall,
                            modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
                        )
                    }
                    items(tasks, key = { it.id }) { task ->
                        TaskRow(
                            task = task,
                            onToggleDone = { viewModel.onIntent(NextActionsIntent.ToggleDone(task.id)) },
                            onClick = { onNavigateToTaskDetail(task.id) },
                        )
                    }
                }
            }
        }
    }
}
