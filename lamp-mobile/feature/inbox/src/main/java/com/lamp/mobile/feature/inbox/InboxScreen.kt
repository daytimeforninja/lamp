package com.lamp.mobile.feature.inbox

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.common.ui.QuickCaptureBar
import com.lamp.mobile.core.common.ui.TaskRow

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun InboxScreen(
    viewModel: InboxViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()

    Column(modifier = Modifier.fillMaxSize()) {
        TopAppBar(title = { Text("Inbox") })

        QuickCaptureBar(
            value = state.inputText,
            onValueChange = { viewModel.onIntent(InboxIntent.InputChanged(it)) },
            onSubmit = { viewModel.onIntent(InboxIntent.Submit) },
        )

        if (state.tasks.isEmpty()) {
            Box(
                modifier = Modifier.fillMaxSize(),
                contentAlignment = androidx.compose.ui.Alignment.Center,
            ) {
                Text(
                    "Inbox zero!",
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        } else {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                items(state.tasks, key = { it.id }) { task ->
                    TaskRow(
                        task = task,
                        onToggleDone = { viewModel.onIntent(InboxIntent.ToggleDone(task.id)) },
                        onClick = { /* TODO: navigate to task detail */ },
                    )
                }
            }
        }
    }
}
