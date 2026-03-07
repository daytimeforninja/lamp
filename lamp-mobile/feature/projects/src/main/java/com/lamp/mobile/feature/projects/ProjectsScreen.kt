package com.lamp.mobile.feature.projects

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.common.ui.QuickCaptureBar

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ProjectsScreen(
    viewModel: ProjectsViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()

    if (state.selectedProject != null) {
        ProjectDetailScreen(
            project = state.selectedProject!!,
            onBack = { viewModel.onIntent(ProjectsIntent.ClearSelection) },
            onUpdatePurpose = { viewModel.onIntent(ProjectsIntent.UpdatePurpose(state.selectedProject!!.id, it)) },
            onUpdateOutcome = { viewModel.onIntent(ProjectsIntent.UpdateOutcome(state.selectedProject!!.id, it)) },
            onUpdateBrainstorm = { viewModel.onIntent(ProjectsIntent.UpdateBrainstorm(state.selectedProject!!.id, it)) },
        )
        return
    }

    Column(modifier = Modifier.fillMaxSize()) {
        TopAppBar(title = { Text("Projects") })

        QuickCaptureBar(
            value = state.inputText,
            onValueChange = { viewModel.onIntent(ProjectsIntent.InputChanged(it)) },
            onSubmit = { viewModel.onIntent(ProjectsIntent.Submit) },
            placeholder = "New project...",
        )

        LazyColumn(modifier = Modifier.fillMaxSize()) {
            items(state.projects, key = { it.id }) { project ->
                val (done, total) = project.completionRatio()
                ListItem(
                    headlineContent = { Text(project.name) },
                    supportingContent = {
                        Text("$done/$total tasks complete")
                    },
                    leadingContent = {
                        if (project.isStuck()) {
                            Icon(Icons.Filled.Warning, "Stuck", tint = MaterialTheme.colorScheme.error)
                        }
                    },
                    trailingContent = {
                        if (total > 0) {
                            CircularProgressIndicator(
                                progress = { done.toFloat() / total },
                                modifier = Modifier.size(24.dp),
                                strokeWidth = 3.dp,
                            )
                        }
                    },
                    modifier = Modifier.clickable {
                        viewModel.onIntent(ProjectsIntent.SelectProject(project))
                    },
                )
                HorizontalDivider()
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ProjectDetailScreen(
    project: com.lamp.mobile.core.model.Project,
    onBack: () -> Unit,
    onUpdatePurpose: (String) -> Unit,
    onUpdateOutcome: (String) -> Unit,
    onUpdateBrainstorm: (String) -> Unit,
) {
    Column(modifier = Modifier.fillMaxSize()) {
        TopAppBar(
            title = { Text(project.name) },
            navigationIcon = {
                IconButton(onClick = onBack) {
                    Icon(Icons.Filled.ArrowBack, "Back")
                }
            },
        )

        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            item {
                OutlinedTextField(
                    value = project.purpose,
                    onValueChange = onUpdatePurpose,
                    label = { Text("Purpose / Why") },
                    modifier = Modifier.fillMaxWidth(),
                    minLines = 2,
                )
            }
            item {
                OutlinedTextField(
                    value = project.outcome,
                    onValueChange = onUpdateOutcome,
                    label = { Text("Desired Outcome") },
                    modifier = Modifier.fillMaxWidth(),
                    minLines = 2,
                )
            }
            item {
                OutlinedTextField(
                    value = project.brainstorm,
                    onValueChange = onUpdateBrainstorm,
                    label = { Text("Brainstorm") },
                    modifier = Modifier.fillMaxWidth(),
                    minLines = 4,
                )
            }
            item {
                Text("Tasks", style = MaterialTheme.typography.titleSmall)
            }
            items(project.tasks, key = { it.id }) { task ->
                com.lamp.mobile.core.common.ui.TaskRow(
                    task = task,
                    onToggleDone = { /* handled by parent */ },
                    onClick = { /* task detail */ },
                    showProject = false,
                )
            }
        }
    }
}
