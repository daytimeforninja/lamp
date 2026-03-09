@file:OptIn(ExperimentalMaterial3Api::class)

package com.lamp.mobile.feature.review

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material.icons.outlined.Circle
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.common.ui.SyncPullRefreshBox
import java.time.format.DateTimeFormatter

private val REVIEW_STEPS = listOf(
    "Clear inbox to zero",
    "Review Next Actions list",
    "Review Waiting For list",
    "Review Projects for next actions",
    "Review Someday/Maybe list",
    "Review past week calendar",
    "Review upcoming calendar",
    "Review Habits and streaks",
    "Process notes and ideas",
    "Review goals and vision",
)

@Composable
fun ReviewScreen(
    viewModel: ReviewViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()
    val progress = state.completedSteps.size.toFloat() / REVIEW_STEPS.size

    SyncPullRefreshBox(modifier = Modifier.fillMaxSize()) {
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(bottom = 32.dp),
        ) {
            item {
                TopAppBar(title = { Text("Weekly Review") })
            }

            item {
                LinearProgressIndicator(
                    progress = { progress },
                    modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp),
                )
                Text(
                    "${state.completedSteps.size} / ${REVIEW_STEPS.size} complete",
                    style = MaterialTheme.typography.labelMedium,
                    modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp),
                )
            }

            // Phase 1: Get Clear
            item {
                Text(
                    "Get Clear",
                    style = MaterialTheme.typography.titleMedium,
                    modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
                )
            }

            item {
                ReviewStepRow(
                    step = REVIEW_STEPS[0],
                    isDone = 0 in state.completedSteps,
                    onToggle = { viewModel.onIntent(ReviewIntent.ToggleStep(0)) },
                    badge = { BadgeCount("${state.inboxCount} items", state.inboxCount > 0) },
                )
            }

            // Phase 2: Get Current
            item {
                Text(
                    "Get Current",
                    style = MaterialTheme.typography.titleMedium,
                    modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
                )
            }

            item {
                ReviewStepRow(
                    step = REVIEW_STEPS[1],
                    isDone = 1 in state.completedSteps,
                    onToggle = { viewModel.onIntent(ReviewIntent.ToggleStep(1)) },
                    badge = { BadgeCount("${state.nextActionsCount} actions", false) },
                )
            }

            item {
                ReviewStepRow(
                    step = REVIEW_STEPS[2],
                    isDone = 2 in state.completedSteps,
                    onToggle = { viewModel.onIntent(ReviewIntent.ToggleStep(2)) },
                    badge = { BadgeCount("${state.waitingTasks.size} waiting", false) },
                )
            }

            // Waiting tasks detail
            if (state.waitingTasks.isNotEmpty()) {
                items(state.waitingTasks) { info ->
                    ListItem(
                        headlineContent = { Text(info.task.title, style = MaterialTheme.typography.bodySmall) },
                        supportingContent = {
                            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                                info.task.waitingFor?.let { Text("For: $it", style = MaterialTheme.typography.labelSmall) }
                                Text("${info.ageDays}d old", style = MaterialTheme.typography.labelSmall)
                                info.task.followUp?.let {
                                    Text("F/U: ${it.format(DateTimeFormatter.ISO_LOCAL_DATE)}",
                                        style = MaterialTheme.typography.labelSmall)
                                }
                            }
                        },
                        modifier = Modifier.padding(start = 32.dp),
                    )
                }
            }

            item {
                ReviewStepRow(
                    step = REVIEW_STEPS[3],
                    isDone = 3 in state.completedSteps,
                    onToggle = { viewModel.onIntent(ReviewIntent.ToggleStep(3)) },
                    badge = if (state.stuckProjects.isNotEmpty()) {
                        {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                Icon(Icons.Filled.Warning, null, tint = MaterialTheme.colorScheme.error,
                                    modifier = Modifier.size(14.dp))
                                Spacer(Modifier.width(4.dp))
                                Text("${state.stuckProjects.size} stuck", style = MaterialTheme.typography.labelSmall,
                                    color = MaterialTheme.colorScheme.error)
                            }
                        }
                    } else null,
                )
            }

            // Stuck projects
            if (state.stuckProjects.isNotEmpty()) {
                items(state.stuckProjects) { project ->
                    ListItem(
                        headlineContent = { Text(project.name, style = MaterialTheme.typography.bodySmall) },
                        leadingContent = {
                            Icon(Icons.Filled.Warning, null, tint = MaterialTheme.colorScheme.error,
                                modifier = Modifier.size(16.dp))
                        },
                        modifier = Modifier.padding(start = 32.dp),
                    )
                }
            }

            item {
                ReviewStepRow(
                    step = REVIEW_STEPS[4],
                    isDone = 4 in state.completedSteps,
                    onToggle = { viewModel.onIntent(ReviewIntent.ToggleStep(4)) },
                    badge = { BadgeCount("${state.somedayCount} items", false) },
                )
            }

            // Steps 5-7
            (5..7).forEach { idx ->
                item {
                    ReviewStepRow(
                        step = REVIEW_STEPS[idx],
                        isDone = idx in state.completedSteps,
                        onToggle = { viewModel.onIntent(ReviewIntent.ToggleStep(idx)) },
                    )
                }
            }

            // Phase 3: Get Creative
            item {
                Text(
                    "Get Creative",
                    style = MaterialTheme.typography.titleMedium,
                    modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
                )
            }

            (8..9).forEach { idx ->
                item {
                    ReviewStepRow(
                        step = REVIEW_STEPS[idx],
                        isDone = idx in state.completedSteps,
                        onToggle = { viewModel.onIntent(ReviewIntent.ToggleStep(idx)) },
                    )
                }
            }

            // Upcoming events summary
            if (state.upcomingEvents.isNotEmpty()) {
                item {
                    Text(
                        "Upcoming Events (14 days)",
                        style = MaterialTheme.typography.titleSmall,
                        modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
                    )
                }
                items(state.upcomingEvents.take(10)) { event ->
                    ListItem(
                        headlineContent = { Text(event.title, style = MaterialTheme.typography.bodySmall) },
                        supportingContent = {
                            val dateStr = event.start.format(DateTimeFormatter.ofPattern("MMM d, HH:mm"))
                            Text(dateStr, style = MaterialTheme.typography.labelSmall)
                        },
                    )
                }
            }

            // Habit stats table
            if (state.habitStats.isNotEmpty()) {
                item {
                    Text(
                        "Habit Stats",
                        style = MaterialTheme.typography.titleSmall,
                        modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
                    )
                }
                items(state.habitStats) { stat ->
                    ListItem(
                        headlineContent = { Text(stat.name, style = MaterialTheme.typography.bodySmall) },
                        trailingContent = {
                            Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                                Text("${stat.thisWeek}/7", style = MaterialTheme.typography.labelMedium)
                                Text("Streak: ${stat.streak}", style = MaterialTheme.typography.labelSmall)
                            }
                        },
                    )
                }
            }
        }
    }
}

@Composable
private fun ReviewStepRow(
    step: String,
    isDone: Boolean,
    onToggle: () -> Unit,
    badge: @Composable (() -> Unit)? = null,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        IconButton(onClick = onToggle, modifier = Modifier.size(24.dp)) {
            Icon(
                if (isDone) Icons.Filled.CheckCircle else Icons.Outlined.Circle,
                "Toggle",
                tint = if (isDone) MaterialTheme.colorScheme.primary
                else MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        Spacer(Modifier.width(12.dp))
        Text(step, style = MaterialTheme.typography.bodyMedium, modifier = Modifier.weight(1f))
        badge?.invoke()
    }
    HorizontalDivider(modifier = Modifier.padding(start = 52.dp))
}

@Composable
private fun BadgeCount(text: String, isWarning: Boolean) {
    Text(
        text,
        style = MaterialTheme.typography.labelSmall,
        color = if (isWarning) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurfaceVariant,
    )
}
