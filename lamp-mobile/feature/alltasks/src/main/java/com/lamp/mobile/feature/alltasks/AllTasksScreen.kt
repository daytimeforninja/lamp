@file:OptIn(ExperimentalMaterial3Api::class)

package com.lamp.mobile.feature.alltasks

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowDownward
import androidx.compose.material.icons.filled.ArrowUpward
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.common.ui.SyncPullRefreshBox
import com.lamp.mobile.core.common.ui.TaskRow

@Composable
fun AllTasksScreen(
    onNavigateToTaskDetail: (java.util.UUID) -> Unit = {},
    viewModel: AllTasksViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()

    val filtered = if (state.searchQuery.isBlank()) state.tasks
    else state.tasks.filter { it.title.contains(state.searchQuery, ignoreCase = true) ||
            it.contexts.any { c -> c.contains(state.searchQuery, ignoreCase = true) } }

    val sorted = AllTasksViewModel.sortTasks(filtered, state.sortColumn, state.sortAscending)

    SyncPullRefreshBox(modifier = Modifier.fillMaxSize()) {
        Column(modifier = Modifier.fillMaxSize()) {
            TopAppBar(title = { Text("All Tasks") })

            // Search bar
            OutlinedTextField(
                value = state.searchQuery,
                onValueChange = { viewModel.onIntent(AllTasksIntent.SetSearchQuery(it)) },
                placeholder = { Text("Search tasks...") },
                leadingIcon = { Icon(Icons.Filled.Search, "Search") },
                modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 4.dp),
                singleLine = true,
            )

            // Sort header
            Row(
                modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 4.dp),
                horizontalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                SortColumn.entries.forEach { col ->
                    val isActive = state.sortColumn == col
                    FilterChip(
                        selected = isActive,
                        onClick = { viewModel.onIntent(AllTasksIntent.SetSort(col)) },
                        label = { Text(col.name.take(5), style = MaterialTheme.typography.labelSmall) },
                        trailingIcon = if (isActive) {
                            {
                                Icon(
                                    if (state.sortAscending) Icons.Filled.ArrowUpward else Icons.Filled.ArrowDownward,
                                    "Sort direction",
                                    modifier = Modifier.size(14.dp),
                                )
                            }
                        } else null,
                    )
                }
            }

            Text(
                "${sorted.size} tasks",
                style = MaterialTheme.typography.labelSmall,
                modifier = Modifier.padding(horizontal = 16.dp),
            )

            LazyColumn(modifier = Modifier.fillMaxSize()) {
                items(sorted, key = { it.id }) { task ->
                    TaskRow(
                        task = task,
                        onToggleDone = { viewModel.onIntent(AllTasksIntent.ToggleDone(task.id)) },
                        onClick = { onNavigateToTaskDetail(task.id) },
                    )
                }
            }
        }
    }
}
