@file:OptIn(ExperimentalMaterial3Api::class)

package com.lamp.mobile.feature.agenda

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.common.ui.SyncPullRefreshBox
import com.lamp.mobile.core.common.ui.TaskRow
import java.time.format.DateTimeFormatter

@Composable
fun AgendaScreen(
    onNavigateToTaskDetail: (java.util.UUID) -> Unit = {},
    viewModel: AgendaViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()

    SyncPullRefreshBox(modifier = Modifier.fillMaxSize()) {
        Column(modifier = Modifier.fillMaxSize()) {
            TopAppBar(title = { Text("Agenda") })

            LazyColumn(
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(bottom = 16.dp),
            ) {
                // Overdue section
                if (state.overdueTasks.isNotEmpty()) {
                    item {
                        Surface(
                            color = MaterialTheme.colorScheme.errorContainer,
                            modifier = Modifier.fillMaxWidth(),
                        ) {
                            Row(
                                modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
                                verticalAlignment = Alignment.CenterVertically,
                            ) {
                                Icon(Icons.Filled.Warning, "Overdue", tint = MaterialTheme.colorScheme.error)
                                Spacer(Modifier.width(8.dp))
                                Text(
                                    "Overdue (${state.overdueTasks.size})",
                                    style = MaterialTheme.typography.titleSmall,
                                    color = MaterialTheme.colorScheme.onErrorContainer,
                                )
                            }
                        }
                    }
                    items(state.overdueTasks, key = { "overdue-${it.id}" }) { task ->
                        TaskRow(
                            task = task,
                            onToggleDone = { viewModel.onIntent(AgendaIntent.ToggleDone(task.id)) },
                            onClick = { onNavigateToTaskDetail(task.id) },
                            trailing = {
                                task.scheduled?.let {
                                    Text("S: $it", style = MaterialTheme.typography.labelSmall,
                                        color = MaterialTheme.colorScheme.error)
                                }
                                task.deadline?.let {
                                    Text("D: $it", style = MaterialTheme.typography.labelSmall,
                                        color = MaterialTheme.colorScheme.error,
                                        modifier = Modifier.padding(start = 4.dp))
                                }
                            },
                        )
                    }
                }

                // Day sections
                state.daySections.forEach { section ->
                    // Sticky header
                    item(key = "header-${section.date}") {
                        Surface(
                            color = MaterialTheme.colorScheme.surfaceVariant,
                            modifier = Modifier.fillMaxWidth(),
                        ) {
                            Text(
                                section.label,
                                style = MaterialTheme.typography.titleSmall,
                                modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
                            )
                        }
                    }

                    // Events
                    items(section.events, key = { "event-${it.id}" }) { event ->
                        ListItem(
                            headlineContent = { Text(event.title) },
                            supportingContent = {
                                if (event.allDay) Text("All day")
                                else Text("${event.start.format(DateTimeFormatter.ofPattern("HH:mm"))} - ${event.end.format(DateTimeFormatter.ofPattern("HH:mm"))}")
                            },
                            leadingContent = {
                                Surface(
                                    color = MaterialTheme.colorScheme.tertiaryContainer,
                                    shape = MaterialTheme.shapes.small,
                                ) {
                                    Text("E", modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
                                        style = MaterialTheme.typography.labelSmall)
                                }
                            },
                        )
                    }

                    // Scheduled tasks
                    items(section.scheduledTasks, key = { "sched-${it.id}" }) { task ->
                        TaskRow(
                            task = task,
                            onToggleDone = { viewModel.onIntent(AgendaIntent.ToggleDone(task.id)) },
                            onClick = { onNavigateToTaskDetail(task.id) },
                            trailing = {
                                Text("Scheduled", style = MaterialTheme.typography.labelSmall,
                                    color = MaterialTheme.colorScheme.primary)
                            },
                        )
                    }

                    // Deadline tasks
                    items(section.deadlineTasks, key = { "dead-${it.id}" }) { task ->
                        TaskRow(
                            task = task,
                            onToggleDone = { viewModel.onIntent(AgendaIntent.ToggleDone(task.id)) },
                            onClick = { onNavigateToTaskDetail(task.id) },
                            trailing = {
                                Text("Deadline", style = MaterialTheme.typography.labelSmall,
                                    color = MaterialTheme.colorScheme.error)
                            },
                        )
                    }

                    // Habits due
                    items(section.habitsDue, key = { "habit-${it.task.id}" }) { habit ->
                        ListItem(
                            headlineContent = { Text(habit.task.title) },
                            supportingContent = { Text("Streak: ${habit.streak} days") },
                            trailingContent = {
                                FilledTonalButton(onClick = {
                                    viewModel.onIntent(AgendaIntent.CompleteHabit(habit.task.id))
                                }) {
                                    Icon(Icons.Filled.Check, "Complete")
                                    Spacer(Modifier.width(4.dp))
                                    Text("Done")
                                }
                            },
                        )
                    }
                }

                if (state.overdueTasks.isEmpty() && state.daySections.isEmpty()) {
                    item {
                        Box(
                            modifier = Modifier.fillMaxWidth().padding(48.dp),
                            contentAlignment = Alignment.Center,
                        ) {
                            Text(
                                "Nothing scheduled",
                                style = MaterialTheme.typography.bodyLarge,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                    }
                }
            }
        }
    }
}
