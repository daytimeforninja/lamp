package com.lamp.mobile.feature.dailyplanning

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Remove
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.common.ui.SpoonMeter
import com.lamp.mobile.core.common.ui.TaskRow

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DailyPlanningScreen(
    viewModel: DailyPlanningViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()
    val plan = state.dayPlan

    Column(modifier = Modifier.fillMaxSize()) {
        TopAppBar(title = { Text("Daily Planning") })

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

            // Available tasks
            item {
                Text("Confirm Tasks", style = MaterialTheme.typography.titleSmall)
            }

            val suggested = state.availableTasks.filter { task ->
                plan.activeContexts.isEmpty() || task.contexts.any { it in plan.activeContexts }
            }

            items(suggested, key = { it.id }) { task ->
                val confirmed = task.id in plan.confirmedTaskIds
                TaskRow(
                    task = task,
                    onToggleDone = {
                        if (confirmed) viewModel.onIntent(DailyPlanningIntent.UnconfirmTask(task.id))
                        else viewModel.onIntent(DailyPlanningIntent.ConfirmTask(task.id))
                    },
                    onClick = { },
                )
            }
        }
    }
}
