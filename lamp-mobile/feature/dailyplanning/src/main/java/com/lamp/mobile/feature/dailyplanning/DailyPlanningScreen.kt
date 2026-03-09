@file:OptIn(ExperimentalLayoutApi::class, ExperimentalMaterial3Api::class)

package com.lamp.mobile.feature.dailyplanning

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Remove
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.common.ui.SpoonMeter
import com.lamp.mobile.core.common.ui.SyncPullRefreshBox
import com.lamp.mobile.core.common.ui.TaskRow

@Composable
fun DailyPlanningScreen(
    onNavigateToTaskDetail: (java.util.UUID) -> Unit = {},
    viewModel: DailyPlanningViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()
    val plan = state.dayPlan

    SyncPullRefreshBox(modifier = Modifier.fillMaxSize()) {
    Column(modifier = Modifier.fillMaxSize()) {
        TopAppBar(title = { Text("Daily Planning") })

        // Stale plan warning
        if (state.isStale) {
            Surface(
                color = MaterialTheme.colorScheme.errorContainer,
                modifier = Modifier.fillMaxWidth(),
            ) {
                Row(
                    modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Icon(Icons.Filled.Warning, null, tint = MaterialTheme.colorScheme.error)
                    Spacer(Modifier.width(8.dp))
                    Text("Plan is from ${plan.date} — not today",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onErrorContainer,
                        modifier = Modifier.weight(1f))
                    TextButton(onClick = { viewModel.onIntent(DailyPlanningIntent.StartNewPlan) }) {
                        Text("Start New")
                    }
                }
            }
        }

        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            // Spoon budget
            item {
                Card(modifier = Modifier.fillMaxWidth()) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Text("Spoon Budget", style = MaterialTheme.typography.titleSmall)
                        Spacer(Modifier.height(8.dp))
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            IconButton(onClick = {
                                viewModel.onIntent(DailyPlanningIntent.SetBudget((plan.spoonBudget - 10).coerceAtLeast(10)))
                            }) { Icon(Icons.Filled.Remove, "Decrease") }
                            Text("${plan.spoonBudget}", style = MaterialTheme.typography.headlineMedium,
                                modifier = Modifier.padding(horizontal = 16.dp))
                            IconButton(onClick = {
                                viewModel.onIntent(DailyPlanningIntent.SetBudget(plan.spoonBudget + 10))
                            }) { Icon(Icons.Filled.Add, "Increase") }
                        }
                        SpoonMeter(spent = plan.spentSpoons, budget = plan.spoonBudget)
                    }
                }
            }

            // Context selection
            item {
                Text("Contexts", style = MaterialTheme.typography.titleSmall)
                FlowRow(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                    state.allContexts.forEach { ctx ->
                        FilterChip(
                            selected = ctx in plan.activeContexts,
                            onClick = { viewModel.onIntent(DailyPlanningIntent.ToggleContext(ctx)) },
                            label = { Text(ctx) },
                        )
                    }
                }
            }

            // Available tasks (with smart filtering)
            item {
                Text("Confirm Tasks", style = MaterialTheme.typography.titleSmall)
            }

            val remaining = plan.remainingBudget()
            val suggested = state.availableTasks
                .filter { task ->
                    val taskEsc = task.esc
                    task.id !in state.rejectedIds &&
                        task.id !in plan.confirmedTaskIds &&
                        (plan.activeContexts.isEmpty() || task.contexts.any { it in plan.activeContexts }) &&
                        (taskEsc == null || taskEsc <= remaining)
                }
                .sortedWith(compareBy<com.lamp.mobile.core.model.Task> {
                    // Overdue first
                    val today = java.time.LocalDate.now()
                    val sched = it.scheduled
                    val dead = it.deadline
                    if (sched != null && sched.isBefore(today)) 0
                    else if (dead != null && dead.isBefore(today)) 0
                    else 1
                }.thenBy { it.priority?.ordinal ?: Int.MAX_VALUE }
                    .thenBy { it.esc ?: Int.MAX_VALUE })

            // Show confirmed tasks in a distinct card
            val confirmedTasks = state.availableTasks.filter { it.id in plan.confirmedTaskIds }
            if (confirmedTasks.isNotEmpty()) {
                item {
                    Card(
                        colors = CardDefaults.cardColors(
                            containerColor = MaterialTheme.colorScheme.primaryContainer,
                        ),
                        modifier = Modifier.fillMaxWidth(),
                    ) {
                        Column(modifier = Modifier.padding(12.dp)) {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                Icon(
                                    Icons.Filled.CheckCircle,
                                    contentDescription = null,
                                    tint = MaterialTheme.colorScheme.primary,
                                    modifier = Modifier.size(20.dp),
                                )
                                Spacer(Modifier.width(8.dp))
                                Text(
                                    "Confirmed (${confirmedTasks.size})",
                                    style = MaterialTheme.typography.titleSmall,
                                    color = MaterialTheme.colorScheme.onPrimaryContainer,
                                )
                                val totalEsc = confirmedTasks.sumOf { it.esc ?: 0 }
                                if (totalEsc > 0) {
                                    Spacer(Modifier.width(8.dp))
                                    Text(
                                        "$totalEsc spoons",
                                        style = MaterialTheme.typography.labelSmall,
                                        color = MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.7f),
                                    )
                                }
                            }
                            Spacer(Modifier.height(8.dp))
                            confirmedTasks.forEach { task ->
                                Row(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .padding(vertical = 4.dp),
                                    verticalAlignment = Alignment.CenterVertically,
                                ) {
                                    Text(
                                        task.title,
                                        style = MaterialTheme.typography.bodyMedium,
                                        color = MaterialTheme.colorScheme.onPrimaryContainer,
                                        modifier = Modifier.weight(1f),
                                    )
                                    val esc = task.esc
                                    if (esc != null) {
                                        Text(
                                            "${esc}sp",
                                            style = MaterialTheme.typography.labelSmall,
                                            color = MaterialTheme.colorScheme.onPrimaryContainer.copy(alpha = 0.6f),
                                        )
                                        Spacer(Modifier.width(8.dp))
                                    }
                                    IconButton(
                                        onClick = { viewModel.onIntent(DailyPlanningIntent.UnconfirmTask(task.id)) },
                                        modifier = Modifier.size(24.dp),
                                    ) {
                                        Icon(
                                            Icons.Filled.Close, "Remove",
                                            modifier = Modifier.size(16.dp),
                                            tint = MaterialTheme.colorScheme.onPrimaryContainer,
                                        )
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Suggestions header
            if (suggested.isNotEmpty()) {
                item {
                    Text("Suggestions", style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }

            items(suggested, key = { "suggest-${it.id}" }) { task ->
                TaskRow(
                    task = task,
                    onToggleDone = { viewModel.onIntent(DailyPlanningIntent.ConfirmTask(task.id)) },
                    onClick = { onNavigateToTaskDetail(task.id) },
                    trailing = {
                        IconButton(onClick = { viewModel.onIntent(DailyPlanningIntent.RejectSuggestion(task.id)) },
                            modifier = Modifier.size(24.dp)) {
                            Icon(Icons.Filled.Close, "Skip", modifier = Modifier.size(16.dp))
                        }
                    },
                )
            }

            // Media section
            if (state.mediaItems.isNotEmpty()) {
                item {
                    Text("Media", style = MaterialTheme.typography.titleSmall)
                }
                items(state.mediaItems, key = { "media-${it.id}" }) { item ->
                    val picked = item.id in plan.pickedMediaIds
                    Row(
                        modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Checkbox(
                            checked = picked,
                            onCheckedChange = {
                                if (picked) viewModel.onIntent(DailyPlanningIntent.UnpickMediaItem(item.id))
                                else viewModel.onIntent(DailyPlanningIntent.PickMediaItem(item.id))
                            },
                        )
                        Text(item.title, style = MaterialTheme.typography.bodyMedium)
                    }
                }
            }

            // Shopping section
            if (state.shoppingItems.isNotEmpty()) {
                item {
                    Text("Shopping", style = MaterialTheme.typography.titleSmall)
                }
                items(state.shoppingItems, key = { "shop-${it.id}" }) { item ->
                    val picked = item.id in plan.pickedShoppingIds
                    Row(
                        modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Checkbox(
                            checked = picked,
                            onCheckedChange = {
                                if (picked) viewModel.onIntent(DailyPlanningIntent.UnpickShoppingItem(item.id))
                                else viewModel.onIntent(DailyPlanningIntent.PickShoppingItem(item.id))
                            },
                        )
                        Text(item.title, style = MaterialTheme.typography.bodyMedium)
                    }
                }
            }
        }
    }
    }
}
