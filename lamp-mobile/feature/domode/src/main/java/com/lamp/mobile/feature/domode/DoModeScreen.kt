package com.lamp.mobile.feature.domode

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.common.ui.SpoonMeter
import com.lamp.mobile.core.common.ui.SyncPullRefreshBox
import com.lamp.mobile.core.common.ui.TaskRow

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DoModeScreen(
    onNavigateToTaskDetail: (java.util.UUID) -> Unit = {},
    viewModel: DoModeViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()
    val plan = state.dayPlan

    SyncPullRefreshBox(modifier = Modifier.fillMaxSize()) {
    Column(modifier = Modifier.fillMaxSize()) {
        TopAppBar(
            title = { Text("Do Mode") },
            colors = TopAppBarDefaults.topAppBarColors(
                containerColor = MaterialTheme.colorScheme.primaryContainer,
            ),
        )

        SpoonMeter(
            spent = plan.spentSpoons,
            budget = plan.spoonBudget,
            modifier = Modifier.padding(16.dp),
        )

        val hasContent = state.confirmedTasks.isNotEmpty() || state.dueHabits.isNotEmpty() ||
            state.pickedMedia.isNotEmpty() || state.pickedShopping.isNotEmpty()

        if (!hasContent) {
            Box(
                modifier = Modifier.fillMaxSize(),
                contentAlignment = Alignment.Center,
            ) {
                Text("No tasks confirmed for today.\nGo to Daily Planning first.",
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        } else {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                // Tasks
                if (state.confirmedTasks.isNotEmpty()) {
                    item {
                        Text(
                            "${state.confirmedTasks.size} tasks remaining",
                            style = MaterialTheme.typography.titleSmall,
                            modifier = Modifier.padding(16.dp),
                        )
                    }
                    items(state.confirmedTasks, key = { it.id }) { task ->
                        TaskRow(
                            task = task,
                            onToggleDone = { viewModel.onIntent(DoModeIntent.MarkDone(task.id)) },
                            onClick = { onNavigateToTaskDetail(task.id) },
                            showProject = true,
                        )
                    }
                }

                // Completed tasks
                if (plan.completedTasks.isNotEmpty()) {
                    item {
                        Text(
                            "Completed (${plan.completedTasks.size})",
                            style = MaterialTheme.typography.titleSmall,
                            modifier = Modifier.padding(16.dp),
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    items(plan.completedTasks) { ct ->
                        ListItem(
                            headlineContent = {
                                Text(ct.title, style = MaterialTheme.typography.bodyMedium,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant)
                            },
                            trailingContent = {
                                ct.esc?.let { Text("${it}sp", style = MaterialTheme.typography.labelSmall) }
                            },
                        )
                    }
                }

                // Habits
                if (state.dueHabits.isNotEmpty()) {
                    item {
                        Text("Habits", style = MaterialTheme.typography.titleSmall,
                            modifier = Modifier.padding(16.dp))
                    }
                    items(state.dueHabits, key = { "habit-${it.task.id}" }) { habit ->
                        Row(
                            modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 4.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Checkbox(
                                checked = false,
                                onCheckedChange = { viewModel.onIntent(DoModeIntent.CompleteHabit(habit.task.id)) },
                            )
                            Text(habit.task.title, style = MaterialTheme.typography.bodyMedium)
                        }
                    }
                }

                // Picked media
                if (state.pickedMedia.isNotEmpty()) {
                    item {
                        Text("Media", style = MaterialTheme.typography.titleSmall,
                            modifier = Modifier.padding(16.dp))
                    }
                    items(state.pickedMedia, key = { "media-${it.id}" }) { item ->
                        Row(
                            modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 4.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Checkbox(
                                checked = item.done,
                                onCheckedChange = { viewModel.onIntent(DoModeIntent.ToggleMediaDone(item.id)) },
                            )
                            Text(item.title, style = MaterialTheme.typography.bodyMedium)
                        }
                    }
                }

                // Picked shopping
                if (state.pickedShopping.isNotEmpty()) {
                    item {
                        Text("Shopping", style = MaterialTheme.typography.titleSmall,
                            modifier = Modifier.padding(16.dp))
                    }
                    items(state.pickedShopping, key = { "shop-${it.id}" }) { item ->
                        Row(
                            modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 4.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Checkbox(
                                checked = item.done,
                                onCheckedChange = { viewModel.onIntent(DoModeIntent.ToggleShoppingDone(item.id)) },
                            )
                            Text(item.title, style = MaterialTheme.typography.bodyMedium)
                        }
                    }
                }
            }
        }
    }
    }
}
