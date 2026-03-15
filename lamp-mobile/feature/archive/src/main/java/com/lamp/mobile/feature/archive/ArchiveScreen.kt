@file:OptIn(ExperimentalMaterial3Api::class)

package com.lamp.mobile.feature.archive

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowDownward
import androidx.compose.material.icons.filled.ArrowUpward
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Cancel
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.common.ui.SyncPullRefreshBox
import com.lamp.mobile.core.model.Task
import com.lamp.mobile.core.model.TaskState
import java.time.format.DateTimeFormatter

private val dateFormatter = DateTimeFormatter.ofPattern("MMM d, yyyy")

@Composable
fun ArchiveScreen(
    onNavigateToTaskDetail: (java.util.UUID) -> Unit = {},
    viewModel: ArchiveViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()

    val filtered = if (state.searchQuery.isBlank()) state.tasks
    else state.tasks.filter {
        it.title.contains(state.searchQuery, ignoreCase = true) ||
            it.contexts.any { c -> c.contains(state.searchQuery, ignoreCase = true) } ||
            it.project?.contains(state.searchQuery, ignoreCase = true) == true
    }

    val sorted = ArchiveViewModel.sortTasks(filtered, state.sortColumn, state.sortAscending)

    SyncPullRefreshBox(modifier = Modifier.fillMaxSize()) {
        Column(modifier = Modifier.fillMaxSize()) {
            TopAppBar(title = { Text("Archive") })

            OutlinedTextField(
                value = state.searchQuery,
                onValueChange = { viewModel.onIntent(ArchiveIntent.SetSearchQuery(it)) },
                placeholder = { Text("Search archived tasks...") },
                leadingIcon = { Icon(Icons.Filled.Search, "Search") },
                modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 4.dp),
                singleLine = true,
            )

            Row(
                modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 4.dp),
                horizontalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                ArchiveSortColumn.entries.forEach { col ->
                    val isActive = state.sortColumn == col
                    FilterChip(
                        selected = isActive,
                        onClick = { viewModel.onIntent(ArchiveIntent.SetSort(col)) },
                        label = { Text(col.name.take(8), style = MaterialTheme.typography.labelSmall) },
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
                "${sorted.size} archived tasks",
                style = MaterialTheme.typography.labelSmall,
                modifier = Modifier.padding(horizontal = 16.dp),
            )

            LazyColumn(modifier = Modifier.fillMaxSize()) {
                items(sorted, key = { it.id }) { task ->
                    ArchiveTaskRow(
                        task = task,
                        onClick = { onNavigateToTaskDetail(task.id) },
                    )
                }
            }
        }
    }
}

@Composable
private fun ArchiveTaskRow(
    task: Task,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier.fillMaxWidth()) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .clickable(onClick = onClick)
                .padding(horizontal = 16.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                imageVector = if (task.state == TaskState.DONE) Icons.Filled.CheckCircle else Icons.Filled.Cancel,
                contentDescription = task.state.name,
                tint = if (task.state == TaskState.DONE) MaterialTheme.colorScheme.primary
                else MaterialTheme.colorScheme.error,
                modifier = Modifier.size(24.dp),
            )

            Spacer(Modifier.width(12.dp))

            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = task.title,
                    style = MaterialTheme.typography.bodyLarge,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    textDecoration = TextDecoration.LineThrough,
                )

                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    task.completed?.let {
                        Text(
                            text = it.format(dateFormatter),
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    task.project?.let {
                        Text(
                            text = it,
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.primary,
                        )
                    }
                }
            }
        }

        HorizontalDivider(modifier = Modifier.padding(start = 52.dp))
    }
}
