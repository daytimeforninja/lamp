package com.lamp.mobile.feature.someday

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.PlayArrow
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.common.ui.TaskRow

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SomedayScreen(
    viewModel: SomedayViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()

    Column(modifier = Modifier.fillMaxSize()) {
        TopAppBar(title = { Text("Someday / Maybe") })

        LazyColumn(modifier = Modifier.fillMaxSize()) {
            items(state.tasks, key = { it.id }) { task ->
                TaskRow(
                    task = task,
                    onToggleDone = { viewModel.onIntent(SomedayIntent.ToggleDone(task.id)) },
                    onClick = { },
                    trailing = {
                        IconButton(onClick = { viewModel.onIntent(SomedayIntent.Activate(task.id)) }) {
                            Icon(Icons.Outlined.PlayArrow, "Activate")
                        }
                    },
                )
            }
        }
    }
}
