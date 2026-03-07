package com.lamp.mobile.feature.habits

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.common.ui.QuickCaptureBar
import com.lamp.mobile.core.common.ui.StreakChart
import java.time.LocalDate

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HabitsScreen(
    viewModel: HabitsViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()

    Column(modifier = Modifier.fillMaxSize()) {
        TopAppBar(title = { Text("Habits") })

        QuickCaptureBar(
            value = state.inputText,
            onValueChange = { viewModel.onIntent(HabitsIntent.InputChanged(it)) },
            onSubmit = { viewModel.onIntent(HabitsIntent.Submit) },
            placeholder = "New habit...",
        )

        LazyColumn(modifier = Modifier.fillMaxSize()) {
            items(state.habits, key = { it.task.id }) { habit ->
                val isDue = habit.isDue(LocalDate.now())
                Card(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 16.dp, vertical = 4.dp),
                ) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Text(habit.task.title, style = MaterialTheme.typography.bodyLarge)
                            if (isDue) {
                                FilledTonalButton(
                                    onClick = { viewModel.onIntent(HabitsIntent.Complete(habit.task.id)) },
                                ) {
                                    Icon(Icons.Filled.Check, "Complete")
                                    Spacer(Modifier.width(4.dp))
                                    Text("Done")
                                }
                            } else {
                                Text("Done today", style = MaterialTheme.typography.labelMedium,
                                    color = MaterialTheme.colorScheme.primary)
                            }
                        }
                        Spacer(Modifier.height(8.dp))
                        StreakChart(
                            completions = habit.completions,
                            streak = habit.streak,
                            bestStreak = habit.bestStreak,
                        )
                    }
                }
            }
        }
    }
}
