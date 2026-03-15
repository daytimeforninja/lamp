@file:OptIn(ExperimentalMaterial3Api::class, ExperimentalLayoutApi::class)

package com.lamp.mobile.feature.taskdetail

import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Send
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.model.*
import java.time.DayOfWeek
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import java.time.temporal.TemporalAdjusters

@Composable
fun TaskDetailScreen(
    onBack: () -> Unit,
    viewModel: TaskDetailViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()

    LaunchedEffect(Unit) {
        viewModel.effects.collect { effect ->
            when (effect) {
                TaskDetailEffect.Deleted -> onBack()
            }
        }
    }

    if (state.isLoading) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            CircularProgressIndicator()
        }
        return
    }

    val task = state.task
    if (task == null) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text("Task not found")
        }
        return
    }

    var showDeleteDialog by remember { mutableStateOf(false) }
    var showScheduledPicker by remember { mutableStateOf(false) }
    var showDeadlinePicker by remember { mutableStateOf(false) }
    var showScheduledTimePicker by remember { mutableStateOf(false) }
    var showDeadlineTimePicker by remember { mutableStateOf(false) }
    var contextInput by remember { mutableStateOf("") }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Edit Task") },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.Filled.ArrowBack, "Back")
                    }
                },
                actions = {
                    IconButton(onClick = { showDeleteDialog = true }) {
                        Icon(Icons.Filled.Delete, "Delete", tint = MaterialTheme.colorScheme.error)
                    }
                },
            )
        },
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .verticalScroll(rememberScrollState())
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            // Title
            OutlinedTextField(
                value = task.title,
                onValueChange = { viewModel.onIntent(TaskDetailIntent.UpdateTitle(it)) },
                label = { Text("Title") },
                modifier = Modifier.fillMaxWidth(),
                singleLine = true,
            )

            // State
            Text("State", style = MaterialTheme.typography.labelMedium)
            Row(
                horizontalArrangement = Arrangement.spacedBy(4.dp),
                modifier = Modifier.horizontalScroll(rememberScrollState()),
            ) {
                TaskState.entries.forEach { ts ->
                    FilterChip(
                        selected = task.state == ts,
                        onClick = { viewModel.onIntent(TaskDetailIntent.SetState(ts)) },
                        label = { Text(ts.keyword) },
                    )
                }
            }

            // Priority
            Text("Priority", style = MaterialTheme.typography.labelMedium)
            Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                Priority.entries.forEach { p ->
                    FilterChip(
                        selected = task.priority == p,
                        onClick = {
                            viewModel.onIntent(TaskDetailIntent.SetPriority(
                                if (task.priority == p) null else p
                            ))
                        },
                        label = { Text(p.name) },
                    )
                }
                FilterChip(
                    selected = task.priority == null,
                    onClick = { viewModel.onIntent(TaskDetailIntent.SetPriority(null)) },
                    label = { Text("None") },
                )
            }

            // Contexts
            Text("Contexts", style = MaterialTheme.typography.labelMedium)
            FlowRow(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                task.contexts.forEach { ctx ->
                    InputChip(
                        selected = true,
                        onClick = { viewModel.onIntent(TaskDetailIntent.RemoveContext(ctx)) },
                        label = { Text(ctx) },
                        trailingIcon = { Text("×") },
                    )
                }
                state.allContexts.filter { it !in task.contexts }.forEach { ctx ->
                    FilterChip(
                        selected = false,
                        onClick = { viewModel.onIntent(TaskDetailIntent.AddContext(ctx)) },
                        label = { Text(ctx) },
                    )
                }
            }
            Row(verticalAlignment = Alignment.CenterVertically) {
                OutlinedTextField(
                    value = contextInput,
                    onValueChange = { contextInput = it },
                    label = { Text("New context") },
                    modifier = Modifier.weight(1f),
                    singleLine = true,
                )
                Spacer(Modifier.width(8.dp))
                FilledTonalButton(onClick = {
                    val ctx = contextInput.trim()
                    if (ctx.isNotEmpty()) {
                        viewModel.onIntent(TaskDetailIntent.AddContext(ctx))
                        contextInput = ""
                    }
                }) { Text("Add") }
            }

            // Scheduled
            DateSection(
                label = "Scheduled",
                date = task.scheduled,
                time = task.scheduledTime,
                onDatePreset = { viewModel.onIntent(TaskDetailIntent.SetScheduled(it)) },
                onClear = { viewModel.onIntent(TaskDetailIntent.SetScheduled(null)); viewModel.onIntent(TaskDetailIntent.SetScheduledTime(null)) },
                onPickDate = { showScheduledPicker = true },
                onPickTime = { showScheduledTimePicker = true },
                onClearTime = { viewModel.onIntent(TaskDetailIntent.SetScheduledTime(null)) },
            )

            // Deadline
            DateSection(
                label = "Deadline",
                date = task.deadline,
                time = task.deadlineTime,
                onDatePreset = { viewModel.onIntent(TaskDetailIntent.SetDeadline(it)) },
                onClear = { viewModel.onIntent(TaskDetailIntent.SetDeadline(null)); viewModel.onIntent(TaskDetailIntent.SetDeadlineTime(null)) },
                onPickDate = { showDeadlinePicker = true },
                onPickTime = { showDeadlineTimePicker = true },
                onClearTime = { viewModel.onIntent(TaskDetailIntent.SetDeadlineTime(null)) },
            )

            // ESC
            Text("Energy (ESC)", style = MaterialTheme.typography.labelMedium)
            Row(
                horizontalArrangement = Arrangement.spacedBy(4.dp),
                modifier = Modifier.horizontalScroll(rememberScrollState()),
            ) {
                listOf(5, 10, 15, 20, 25, 30, 40, 50, 75, 100).forEach { esc ->
                    FilterChip(
                        selected = task.esc == esc,
                        onClick = {
                            viewModel.onIntent(TaskDetailIntent.SetEsc(
                                if (task.esc == esc) null else esc
                            ))
                        },
                        label = { Text("$esc") },
                    )
                }
                FilterChip(
                    selected = task.esc == null,
                    onClick = { viewModel.onIntent(TaskDetailIntent.SetEsc(null)) },
                    label = { Text("Clear") },
                )
            }

            // Recurrence
            RecurrenceSection(
                recurrence = task.recurrence,
                onSetRecurrence = { viewModel.onIntent(TaskDetailIntent.SetRecurrence(it)) },
            )

            // Project
            ProjectDropdown(
                currentProject = task.project,
                allProjects = state.allProjects,
                onSetProject = { viewModel.onIntent(TaskDetailIntent.SetProject(it)) },
            )

            // Waiting-for (visible when WAITING)
            if (task.state == TaskState.WAITING) {
                WaitingForSection(
                    waitingFor = task.waitingFor,
                    allContacts = state.allContacts,
                    onSetWaitingFor = { viewModel.onIntent(TaskDetailIntent.SetWaitingFor(it)) },
                    followUp = task.followUp,
                    onSetFollowUp = { viewModel.onIntent(TaskDetailIntent.SetFollowUp(it)) },
                )
            }

            // Delegated
            task.delegated?.let {
                Text("Delegated", style = MaterialTheme.typography.labelMedium)
                Text(it.format(DateTimeFormatter.ISO_LOCAL_DATE))
            }

            // Notes
            Text("Notes", style = MaterialTheme.typography.labelMedium)
            if (task.notes.isNotBlank()) {
                Card(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        task.notes,
                        modifier = Modifier.padding(12.dp),
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
            }
            Row(verticalAlignment = Alignment.CenterVertically) {
                OutlinedTextField(
                    value = state.noteInput,
                    onValueChange = { viewModel.onIntent(TaskDetailIntent.NoteInputChanged(it)) },
                    label = { Text("Append note") },
                    modifier = Modifier.weight(1f),
                    minLines = 2,
                )
                Spacer(Modifier.width(4.dp))
                FilledIconButton(onClick = { viewModel.onIntent(TaskDetailIntent.AppendNote) }) {
                    Icon(Icons.Filled.Send, "Append")
                }
            }

            // Delete button
            OutlinedButton(
                onClick = { showDeleteDialog = true },
                colors = ButtonDefaults.outlinedButtonColors(contentColor = MaterialTheme.colorScheme.error),
                modifier = Modifier.fillMaxWidth(),
            ) {
                Icon(Icons.Filled.Delete, null)
                Spacer(Modifier.width(8.dp))
                Text("Delete Task")
            }

            Spacer(Modifier.height(32.dp))
        }
    }

    // Date pickers
    if (showScheduledPicker) {
        LampDatePicker(
            initialDate = task.scheduled,
            onConfirm = {
                viewModel.onIntent(TaskDetailIntent.SetScheduled(it))
                showScheduledPicker = false
            },
            onDismiss = { showScheduledPicker = false },
        )
    }
    if (showDeadlinePicker) {
        LampDatePicker(
            initialDate = task.deadline,
            onConfirm = {
                viewModel.onIntent(TaskDetailIntent.SetDeadline(it))
                showDeadlinePicker = false
            },
            onDismiss = { showDeadlinePicker = false },
        )
    }
    if (showScheduledTimePicker) {
        LampTimePicker(
            onConfirm = { h, m ->
                viewModel.onIntent(TaskDetailIntent.SetScheduledTime("%02d:%02d".format(h, m)))
                showScheduledTimePicker = false
            },
            onDismiss = { showScheduledTimePicker = false },
        )
    }
    if (showDeadlineTimePicker) {
        LampTimePicker(
            onConfirm = { h, m ->
                viewModel.onIntent(TaskDetailIntent.SetDeadlineTime("%02d:%02d".format(h, m)))
                showDeadlineTimePicker = false
            },
            onDismiss = { showDeadlineTimePicker = false },
        )
    }

    // Delete confirmation
    if (showDeleteDialog) {
        AlertDialog(
            onDismissRequest = { showDeleteDialog = false },
            title = { Text("Delete Task?") },
            text = { Text("\"${task.title}\" will be permanently deleted.") },
            confirmButton = {
                TextButton(onClick = {
                    showDeleteDialog = false
                    viewModel.onIntent(TaskDetailIntent.Delete)
                }) { Text("Delete", color = MaterialTheme.colorScheme.error) }
            },
            dismissButton = {
                TextButton(onClick = { showDeleteDialog = false }) { Text("Cancel") }
            },
        )
    }
}

@Composable
private fun DateSection(
    label: String,
    date: LocalDate?,
    time: String?,
    onDatePreset: (LocalDate) -> Unit,
    onClear: () -> Unit,
    onPickDate: () -> Unit,
    onPickTime: () -> Unit,
    onClearTime: () -> Unit,
) {
    val today = LocalDate.now()
    Text(label, style = MaterialTheme.typography.labelMedium)
    if (date != null) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(date.format(DateTimeFormatter.ISO_LOCAL_DATE))
            time?.let {
                Spacer(Modifier.width(8.dp))
                Text(it, style = MaterialTheme.typography.bodySmall)
            }
        }
    }
    Row(
        horizontalArrangement = Arrangement.spacedBy(4.dp),
        modifier = Modifier.horizontalScroll(rememberScrollState()),
    ) {
        AssistChip(onClick = { onDatePreset(today) }, label = { Text("Today") })
        AssistChip(onClick = { onDatePreset(today.plusDays(1)) }, label = { Text("Tmrw") })
        AssistChip(onClick = {
            onDatePreset(today.with(TemporalAdjusters.next(DayOfWeek.MONDAY)))
        }, label = { Text("Mon") })
        AssistChip(onClick = { onDatePreset(today.plusWeeks(1)) }, label = { Text("+1w") })
        AssistChip(onClick = { onDatePreset(today.plusWeeks(2)) }, label = { Text("+2w") })
        AssistChip(onClick = { onDatePreset(today.plusMonths(1)) }, label = { Text("+1m") })
        AssistChip(onClick = onPickDate, label = { Text("Pick") })
        if (date != null) {
            AssistChip(onClick = onPickTime, label = { Text("Time") })
            if (time != null) {
                AssistChip(onClick = onClearTime, label = { Text("×T") })
            }
            AssistChip(onClick = onClear, label = { Text("Clear") })
        }
    }
}

@Composable
private fun RecurrenceSection(
    recurrence: Recurrence?,
    onSetRecurrence: (Recurrence?) -> Unit,
) {
    var count by remember(recurrence) { mutableStateOf(recurrence?.interval?.count?.toString() ?: "1") }
    var unit by remember(recurrence) { mutableStateOf(recurrence?.interval?.unit ?: RecurrenceUnit.DAY) }
    var type by remember(recurrence) {
        mutableStateOf(
            when (recurrence) {
                is Recurrence.Relative -> 1
                is Recurrence.Strict -> 2
                else -> 0
            }
        )
    }

    Text("Recurrence", style = MaterialTheme.typography.labelMedium)
    if (recurrence != null) {
        Text("Current: ${recurrence.toOrgString()}", style = MaterialTheme.typography.bodySmall)
    }

    // Type selector
    Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        listOf("Standard" to 0, "Relative" to 1, "Strict" to 2).forEach { (label, idx) ->
            FilterChip(
                selected = type == idx,
                onClick = { type = idx },
                label = { Text(label) },
            )
        }
    }

    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        OutlinedTextField(
            value = count,
            onValueChange = { count = it.filter { c -> c.isDigit() } },
            label = { Text("N") },
            modifier = Modifier.width(72.dp),
            singleLine = true,
        )
        RecurrenceUnit.entries.forEach { u ->
            FilterChip(
                selected = unit == u,
                onClick = { unit = u },
                label = { Text(u.suffix.uppercase()) },
            )
        }
    }

    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        FilledTonalButton(onClick = {
            val n = count.toIntOrNull() ?: return@FilledTonalButton
            if (n <= 0) return@FilledTonalButton
            val interval = RecurrenceInterval(n, unit)
            val rec = when (type) {
                1 -> Recurrence.Relative(interval)
                2 -> Recurrence.Strict(interval)
                else -> Recurrence.Standard(interval)
            }
            onSetRecurrence(rec)
        }) { Text("Set") }
        if (recurrence != null) {
            OutlinedButton(onClick = { onSetRecurrence(null) }) { Text("Clear") }
        }
    }
}

@Composable
private fun ProjectDropdown(
    currentProject: String?,
    allProjects: List<String>,
    onSetProject: (String?) -> Unit,
) {
    var expanded by remember { mutableStateOf(false) }

    Text("Project", style = MaterialTheme.typography.labelMedium)
    ExposedDropdownMenuBox(expanded = expanded, onExpandedChange = { expanded = it }) {
        OutlinedTextField(
            value = currentProject ?: "(none)",
            onValueChange = {},
            readOnly = true,
            trailingIcon = { ExposedDropdownMenuDefaults.TrailingIcon(expanded) },
            modifier = Modifier.fillMaxWidth().menuAnchor(),
        )
        ExposedDropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
            DropdownMenuItem(
                text = { Text("(none)") },
                onClick = { onSetProject(null); expanded = false },
            )
            allProjects.forEach { name ->
                DropdownMenuItem(
                    text = { Text(name) },
                    onClick = { onSetProject(name); expanded = false },
                )
            }
        }
    }
}

@Composable
private fun WaitingForSection(
    waitingFor: String?,
    allContacts: List<String>,
    onSetWaitingFor: (String?) -> Unit,
    followUp: LocalDate?,
    onSetFollowUp: (LocalDate?) -> Unit,
) {
    var showFollowUpPicker by remember { mutableStateOf(false) }
    var query by remember { mutableStateOf(waitingFor ?: "") }
    var expanded by remember { mutableStateOf(false) }

    Text("Waiting For", style = MaterialTheme.typography.labelMedium)
    ExposedDropdownMenuBox(expanded = expanded, onExpandedChange = { expanded = it }) {
        OutlinedTextField(
            value = query,
            onValueChange = {
                query = it
                expanded = true
                onSetWaitingFor(it.ifBlank { null })
            },
            label = { Text("Person / Contact") },
            modifier = Modifier.fillMaxWidth().menuAnchor(),
            singleLine = true,
        )
        val filtered = allContacts.filter { it.contains(query, ignoreCase = true) }
        if (filtered.isNotEmpty()) {
            ExposedDropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
                filtered.forEach { name ->
                    DropdownMenuItem(
                        text = { Text(name) },
                        onClick = {
                            query = name
                            onSetWaitingFor(name)
                            expanded = false
                        },
                    )
                }
            }
        }
    }

    Text("Follow-up", style = MaterialTheme.typography.labelMedium)
    if (followUp != null) {
        Text(followUp.format(DateTimeFormatter.ISO_LOCAL_DATE))
    }
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        AssistChip(onClick = { showFollowUpPicker = true }, label = { Text("Pick date") })
        if (followUp != null) {
            AssistChip(onClick = { onSetFollowUp(null) }, label = { Text("Clear") })
        }
    }

    if (showFollowUpPicker) {
        LampDatePicker(
            initialDate = followUp,
            onConfirm = { onSetFollowUp(it); showFollowUpPicker = false },
            onDismiss = { showFollowUpPicker = false },
        )
    }
}

@Composable
private fun LampDatePicker(
    initialDate: LocalDate?,
    onConfirm: (LocalDate) -> Unit,
    onDismiss: () -> Unit,
) {
    val datePickerState = rememberDatePickerState(
        initialSelectedDateMillis = initialDate?.let {
            it.toEpochDay() * 86_400_000L
        },
    )
    DatePickerDialog(
        onDismissRequest = onDismiss,
        confirmButton = {
            TextButton(onClick = {
                datePickerState.selectedDateMillis?.let { millis ->
                    val date = LocalDate.ofEpochDay(millis / 86_400_000L)
                    onConfirm(date)
                }
            }) { Text("OK") }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) { Text("Cancel") }
        },
    ) {
        DatePicker(state = datePickerState)
    }
}

@Composable
private fun LampTimePicker(
    onConfirm: (hour: Int, minute: Int) -> Unit,
    onDismiss: () -> Unit,
) {
    val timePickerState = rememberTimePickerState()
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Select Time") },
        text = { TimePicker(state = timePickerState) },
        confirmButton = {
            TextButton(onClick = {
                onConfirm(timePickerState.hour, timePickerState.minute)
            }) { Text("OK") }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) { Text("Cancel") }
        },
    )
}
