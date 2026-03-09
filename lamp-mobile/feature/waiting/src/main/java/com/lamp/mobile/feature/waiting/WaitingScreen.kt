package com.lamp.mobile.feature.waiting

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.common.ui.SyncPullRefreshBox
import com.lamp.mobile.core.common.ui.TaskRow
import java.time.format.DateTimeFormatter

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun WaitingScreen(
    onNavigateToTaskDetail: (java.util.UUID) -> Unit = {},
    viewModel: WaitingViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()

    SyncPullRefreshBox(modifier = Modifier.fillMaxSize()) {
        Column(modifier = Modifier.fillMaxSize()) {
            TopAppBar(title = { Text("Waiting For") })

            LazyColumn(modifier = Modifier.fillMaxSize()) {
                items(state.tasks, key = { it.id }) { task ->
                    TaskRow(
                        task = task,
                        onToggleDone = { viewModel.onIntent(WaitingIntent.ToggleDone(task.id)) },
                        onClick = { onNavigateToTaskDetail(task.id) },
                        trailing = {
                            task.waitingFor?.let {
                                Text(it, style = MaterialTheme.typography.labelSmall,
                                    color = MaterialTheme.colorScheme.tertiary)
                            }
                            task.followUp?.let {
                                Text(
                                    it.format(DateTimeFormatter.ISO_LOCAL_DATE),
                                    style = MaterialTheme.typography.labelSmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                    modifier = Modifier.padding(start = 8.dp),
                                )
                            }
                        },
                    )
                }
            }
        }
    }
}
