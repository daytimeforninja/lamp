package com.lamp.mobile.feature.domode

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.common.ui.SpoonMeter
import com.lamp.mobile.core.common.ui.TaskRow

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DoModeScreen(
    viewModel: DoModeViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()
    val plan = state.dayPlan

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

        if (state.confirmedTasks.isEmpty()) {
            Box(
                modifier = Modifier.fillMaxSize(),
                contentAlignment = androidx.compose.ui.Alignment.Center,
            ) {
                Text("No tasks confirmed for today.\nGo to Daily Planning first.",
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        } else {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
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
                        onClick = { },
                        showProject = true,
                    )
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
            }
        }
    }
}
