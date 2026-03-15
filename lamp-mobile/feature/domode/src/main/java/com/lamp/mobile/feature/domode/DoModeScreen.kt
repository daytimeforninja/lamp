package com.lamp.mobile.feature.domode

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.Stop
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.common.ui.SpoonMeter
import com.lamp.mobile.core.common.ui.SyncPullRefreshBox
import com.lamp.mobile.core.common.ui.TaskRow
import java.time.Duration
import java.time.LocalDateTime

private fun formatDuration(secs: Long): String {
    val h = secs / 3600
    val m = (secs % 3600) / 60
    val s = secs % 60
    return if (h > 0) "${h}h %02dm".format(m) else "${m}m %02ds".format(s)
}

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
                        val isActive = state.activeTaskId == task.id
                        val now = LocalDateTime.now()
                        val activeSecs = if (isActive && state.activeTimerStart != null) {
                            Duration.between(state.activeTimerStart, now).seconds.coerceAtLeast(0)
                        } else 0L
                        // Force recomposition on tick (read the value)
                        @Suppress("UNUSED_VARIABLE")
                        val tick = state.timerTick
                        val totalSecs = task.totalWorkSecs() + activeSecs

                        val containerColor = if (isActive) {
                            MaterialTheme.colorScheme.primaryContainer
                        } else {
                            MaterialTheme.colorScheme.surface
                        }

                        Surface(
                            color = containerColor,
                            modifier = Modifier.fillMaxWidth(),
                        ) {
                            Column {
                                Row(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .clickable { viewModel.onIntent(DoModeIntent.ToggleWorkTimer(task.id)) }
                                        .padding(horizontal = 16.dp, vertical = 12.dp),
                                    verticalAlignment = Alignment.CenterVertically,
                                ) {
                                    // Play/Stop icon
                                    Icon(
                                        imageVector = if (isActive) Icons.Default.Stop else Icons.Default.PlayArrow,
                                        contentDescription = if (isActive) "Stop timer" else "Start timer",
                                        tint = if (isActive) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant,
                                        modifier = Modifier.size(24.dp),
                                    )
                                    Spacer(Modifier.width(12.dp))

                                    // Task title + ESC
                                    Column(modifier = Modifier.weight(1f)) {
                                        Text(
                                            task.title,
                                            style = MaterialTheme.typography.bodyLarge,
                                            fontWeight = if (isActive) FontWeight.Bold else FontWeight.Normal,
                                        )
                                        if (task.project != null) {
                                            Text(
                                                task.project!!,
                                                style = MaterialTheme.typography.labelSmall,
                                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                                            )
                                        }
                                    }

                                    // Timer / total time
                                    Column(horizontalAlignment = Alignment.End) {
                                        if (isActive) {
                                            Text(
                                                formatDuration(activeSecs),
                                                style = MaterialTheme.typography.titleMedium,
                                                color = MaterialTheme.colorScheme.primary,
                                                fontWeight = FontWeight.Bold,
                                            )
                                        }
                                        if (totalSecs > 0) {
                                            Text(
                                                formatDuration(totalSecs),
                                                style = MaterialTheme.typography.labelSmall,
                                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                                            )
                                        }
                                    }

                                    Spacer(Modifier.width(8.dp))

                                    // ESC badge
                                    task.esc?.let { esc ->
                                        Surface(
                                            color = MaterialTheme.colorScheme.tertiaryContainer,
                                            shape = MaterialTheme.shapes.small,
                                        ) {
                                            Text(
                                                "${esc}sp",
                                                modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp),
                                                style = MaterialTheme.typography.labelSmall,
                                            )
                                        }
                                    }

                                    Spacer(Modifier.width(8.dp))

                                    // Done checkbox
                                    Checkbox(
                                        checked = false,
                                        onCheckedChange = { viewModel.onIntent(DoModeIntent.MarkDone(task.id)) },
                                    )
                                }
                                HorizontalDivider()
                            }
                        }
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
