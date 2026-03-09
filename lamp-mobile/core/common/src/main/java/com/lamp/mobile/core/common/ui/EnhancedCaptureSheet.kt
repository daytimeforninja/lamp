@file:OptIn(ExperimentalMaterial3Api::class, ExperimentalLayoutApi::class)

package com.lamp.mobile.core.common.ui

import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.lamp.mobile.core.model.Priority
import com.lamp.mobile.core.model.TaskState
import java.time.DayOfWeek
import java.time.LocalDate
import java.time.temporal.TemporalAdjusters

data class EnhancedCaptureData(
    val title: String = "",
    val state: TaskState = TaskState.TODO,
    val priority: Priority? = null,
    val contexts: List<String> = emptyList(),
    val project: String? = null,
    val scheduled: LocalDate? = null,
    val esc: Int? = null,
)

@Composable
fun EnhancedCaptureSheet(
    onDismiss: () -> Unit,
    onSubmit: (EnhancedCaptureData) -> Unit,
    allContexts: Set<String> = emptySet(),
    allProjects: List<String> = emptyList(),
) {
    var data by remember { mutableStateOf(EnhancedCaptureData()) }
    val today = LocalDate.now()

    ModalBottomSheet(onDismissRequest = onDismiss) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text("Quick Capture", style = MaterialTheme.typography.titleMedium)

            OutlinedTextField(
                value = data.title,
                onValueChange = { data = data.copy(title = it) },
                label = { Text("Title") },
                modifier = Modifier.fillMaxWidth(),
                singleLine = true,
            )

            // State chips
            Text("State", style = MaterialTheme.typography.labelMedium)
            Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                listOf(TaskState.TODO, TaskState.NEXT, TaskState.WAITING, TaskState.SOMEDAY).forEach { ts ->
                    FilterChip(
                        selected = data.state == ts,
                        onClick = { data = data.copy(state = ts) },
                        label = { Text(ts.keyword) },
                    )
                }
            }

            // Priority chips
            Text("Priority", style = MaterialTheme.typography.labelMedium)
            Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                Priority.entries.forEach { p ->
                    FilterChip(
                        selected = data.priority == p,
                        onClick = { data = data.copy(priority = if (data.priority == p) null else p) },
                        label = { Text(p.name) },
                    )
                }
            }

            // Context toggles
            if (allContexts.isNotEmpty()) {
                Text("Contexts", style = MaterialTheme.typography.labelMedium)
                FlowRow(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                    allContexts.forEach { ctx ->
                        FilterChip(
                            selected = ctx in data.contexts,
                            onClick = {
                                data = data.copy(contexts = if (ctx in data.contexts) data.contexts - ctx else data.contexts + ctx)
                            },
                            label = { Text(ctx) },
                        )
                    }
                }
            }

            // Project dropdown
            if (allProjects.isNotEmpty()) {
                var expanded by remember { mutableStateOf(false) }
                ExposedDropdownMenuBox(expanded = expanded, onExpandedChange = { expanded = it }) {
                    OutlinedTextField(
                        value = data.project ?: "(no project)",
                        onValueChange = {},
                        readOnly = true,
                        label = { Text("Project") },
                        trailingIcon = { ExposedDropdownMenuDefaults.TrailingIcon(expanded) },
                        modifier = Modifier.fillMaxWidth().menuAnchor(),
                    )
                    ExposedDropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
                        DropdownMenuItem(
                            text = { Text("(none)") },
                            onClick = { data = data.copy(project = null); expanded = false },
                        )
                        allProjects.forEach { name ->
                            DropdownMenuItem(
                                text = { Text(name) },
                                onClick = { data = data.copy(project = name); expanded = false },
                            )
                        }
                    }
                }
            }

            // Scheduled quick-pick
            Text("Schedule", style = MaterialTheme.typography.labelMedium)
            Row(
                horizontalArrangement = Arrangement.spacedBy(4.dp),
                modifier = Modifier.horizontalScroll(rememberScrollState()),
            ) {
                AssistChip(onClick = { data = data.copy(scheduled = today) }, label = { Text("Today") })
                AssistChip(onClick = { data = data.copy(scheduled = today.plusDays(1)) }, label = { Text("Tmrw") })
                AssistChip(onClick = { data = data.copy(scheduled = today.with(TemporalAdjusters.next(DayOfWeek.MONDAY))) }, label = { Text("Mon") })
                AssistChip(onClick = { data = data.copy(scheduled = today.plusWeeks(1)) }, label = { Text("+1w") })
                if (data.scheduled != null) {
                    AssistChip(onClick = { data = data.copy(scheduled = null) }, label = { Text("Clear") })
                }
            }

            // ESC picker
            Text("Energy", style = MaterialTheme.typography.labelMedium)
            Row(
                horizontalArrangement = Arrangement.spacedBy(4.dp),
                modifier = Modifier.horizontalScroll(rememberScrollState()),
            ) {
                listOf(5, 10, 15, 20, 25, 30, 50).forEach { esc ->
                    FilterChip(
                        selected = data.esc == esc,
                        onClick = { data = data.copy(esc = if (data.esc == esc) null else esc) },
                        label = { Text("$esc") },
                    )
                }
            }

            // Submit
            Button(
                onClick = {
                    if (data.title.isNotBlank()) {
                        onSubmit(data)
                        onDismiss()
                    }
                },
                modifier = Modifier.fillMaxWidth(),
                enabled = data.title.isNotBlank(),
            ) { Text("Add Task") }

            Spacer(Modifier.height(16.dp))
        }
    }
}
