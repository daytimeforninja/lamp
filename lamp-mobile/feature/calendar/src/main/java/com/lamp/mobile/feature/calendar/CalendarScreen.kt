@file:OptIn(ExperimentalMaterial3Api::class)

package com.lamp.mobile.feature.calendar

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.common.ui.SyncPullRefreshBox
import java.time.LocalDate
import java.time.LocalTime
import java.time.format.DateTimeFormatter
import java.time.format.TextStyle
import java.util.Locale

@Composable
fun CalendarScreen(
    viewModel: CalendarViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()

    // Show event form if active
    val eventForm = state.eventForm
    if (eventForm != null) {
        EventFormScreen(
            form = eventForm,
            onTitleChange = { viewModel.onIntent(CalendarIntent.SetEventTitle(it)) },
            onAllDayChange = { viewModel.onIntent(CalendarIntent.SetEventAllDay(it)) },
            onStartDateChange = { viewModel.onIntent(CalendarIntent.SetEventStartDate(it)) },
            onStartTimeChange = { viewModel.onIntent(CalendarIntent.SetEventStartTime(it)) },
            onEndDateChange = { viewModel.onIntent(CalendarIntent.SetEventEndDate(it)) },
            onEndTimeChange = { viewModel.onIntent(CalendarIntent.SetEventEndTime(it)) },
            onLocationChange = { viewModel.onIntent(CalendarIntent.SetEventLocation(it)) },
            onDescriptionChange = { viewModel.onIntent(CalendarIntent.SetEventDescription(it)) },
            onSave = { viewModel.onIntent(CalendarIntent.SubmitEvent) },
            onCancel = { viewModel.onIntent(CalendarIntent.CancelEventForm) },
            onDelete = eventForm.id?.let { id ->
                { viewModel.onIntent(CalendarIntent.DeleteEvent(id)) }
            },
        )
        return
    }

    val month = state.currentMonth
    val firstDay = month.atDay(1)
    val daysInMonth = month.lengthOfMonth()
    val startDow = firstDay.dayOfWeek.value % 7
    val eventDates = state.events.map { it.start.toLocalDate() }.toSet()
    val selectedDayEvents = state.events.filter { it.start.toLocalDate() == state.selectedDate }

    Scaffold(
        floatingActionButton = {
            FloatingActionButton(onClick = { viewModel.onIntent(CalendarIntent.CreateEvent) }) {
                Icon(Icons.Filled.Add, "New event")
            }
        },
    ) { padding ->
        SyncPullRefreshBox(modifier = Modifier.fillMaxSize()) {
        Column(modifier = Modifier.fillMaxSize().padding(padding)) {
            TopAppBar(title = { Text("Calendar") })

            // Month nav
            Row(
                modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                IconButton(onClick = { viewModel.onIntent(CalendarIntent.PrevMonth) }) {
                    Icon(Icons.Filled.ChevronLeft, "Previous month")
                }
                Text(
                    "${month.month.getDisplayName(TextStyle.FULL, Locale.getDefault())} ${month.year}",
                    style = MaterialTheme.typography.titleMedium,
                )
                IconButton(onClick = { viewModel.onIntent(CalendarIntent.NextMonth) }) {
                    Icon(Icons.Filled.ChevronRight, "Next month")
                }
            }

            // Day-of-week headers
            Row(modifier = Modifier.fillMaxWidth().padding(horizontal = 8.dp)) {
                listOf("Su", "Mo", "Tu", "We", "Th", "Fr", "Sa").forEach {
                    Text(
                        it, modifier = Modifier.weight(1f),
                        textAlign = TextAlign.Center,
                        style = MaterialTheme.typography.labelSmall,
                    )
                }
            }

            // Calendar grid
            val cells = (0 until 42).map { idx ->
                val dayNum = idx - startDow + 1
                if (dayNum in 1..daysInMonth) dayNum else null
            }

            LazyVerticalGrid(
                columns = GridCells.Fixed(7),
                modifier = Modifier.fillMaxWidth().height(240.dp).padding(horizontal = 8.dp),
                userScrollEnabled = false,
            ) {
                items(cells) { dayNum ->
                    if (dayNum != null) {
                        val date = month.atDay(dayNum)
                        val isSelected = date == state.selectedDate
                        val hasEvents = date in eventDates
                        val isToday = date == LocalDate.now()

                        Box(
                            modifier = Modifier
                                .aspectRatio(1f)
                                .clip(CircleShape)
                                .then(
                                    if (isSelected) Modifier.background(MaterialTheme.colorScheme.primaryContainer, CircleShape)
                                    else if (isToday) Modifier.background(MaterialTheme.colorScheme.surfaceVariant, CircleShape)
                                    else Modifier
                                )
                                .clickable { viewModel.onIntent(CalendarIntent.SelectDay(date)) },
                            contentAlignment = Alignment.Center,
                        ) {
                            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                                Text("$dayNum", style = MaterialTheme.typography.bodySmall)
                                if (hasEvents) {
                                    Box(
                                        Modifier.size(4.dp).clip(CircleShape)
                                            .background(MaterialTheme.colorScheme.primary)
                                    )
                                }
                            }
                        }
                    } else {
                        Spacer(Modifier.aspectRatio(1f))
                    }
                }
            }

            HorizontalDivider()

            // Events for selected day
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                if (selectedDayEvents.isEmpty()) {
                    item {
                        Text(
                            "No events",
                            modifier = Modifier.padding(16.dp),
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
                items(selectedDayEvents, key = { it.id }) { event ->
                    ListItem(
                        headlineContent = { Text(event.title) },
                        supportingContent = {
                            if (event.allDay) Text("All day")
                            else Text("${event.start.format(DateTimeFormatter.ofPattern("HH:mm"))} - ${event.end.format(DateTimeFormatter.ofPattern("HH:mm"))}")
                        },
                        overlineContent = if (event.calendarName.isNotBlank()) {
                            { Text(event.calendarName) }
                        } else null,
                        modifier = Modifier.clickable {
                            viewModel.onIntent(CalendarIntent.EditEvent(event.id))
                        },
                    )
                    HorizontalDivider()
                }
            }
        }
        }
    }
}

@Composable
private fun EventFormScreen(
    form: EventFormState,
    onTitleChange: (String) -> Unit,
    onAllDayChange: (Boolean) -> Unit,
    onStartDateChange: (LocalDate) -> Unit,
    onStartTimeChange: (LocalTime) -> Unit,
    onEndDateChange: (LocalDate) -> Unit,
    onEndTimeChange: (LocalTime) -> Unit,
    onLocationChange: (String) -> Unit,
    onDescriptionChange: (String) -> Unit,
    onSave: () -> Unit,
    onCancel: () -> Unit,
    onDelete: (() -> Unit)?,
) {
    var showStartDatePicker by remember { mutableStateOf(false) }
    var showEndDatePicker by remember { mutableStateOf(false) }
    var showStartTimePicker by remember { mutableStateOf(false) }
    var showEndTimePicker by remember { mutableStateOf(false) }
    var showDeleteDialog by remember { mutableStateOf(false) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(if (form.id != null) "Edit Event" else "New Event") },
                navigationIcon = {
                    IconButton(onClick = onCancel) { Icon(Icons.Filled.Close, "Cancel") }
                },
                actions = {
                    TextButton(onClick = onSave) { Text("Save") }
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
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            OutlinedTextField(
                value = form.title,
                onValueChange = onTitleChange,
                label = { Text("Title") },
                modifier = Modifier.fillMaxWidth(),
                singleLine = true,
            )

            form.validationError?.let {
                Text(it, color = MaterialTheme.colorScheme.error, style = MaterialTheme.typography.bodySmall)
            }

            Row(verticalAlignment = Alignment.CenterVertically) {
                Text("All day", modifier = Modifier.weight(1f))
                Switch(checked = form.allDay, onCheckedChange = onAllDayChange)
            }

            // Start date
            OutlinedTextField(
                value = form.startDate.format(DateTimeFormatter.ISO_LOCAL_DATE),
                onValueChange = {},
                readOnly = true,
                label = { Text("Start date") },
                modifier = Modifier.fillMaxWidth().clickable { showStartDatePicker = true },
                trailingIcon = {
                    IconButton(onClick = { showStartDatePicker = true }) {
                        Icon(Icons.Filled.CalendarMonth, "Pick date")
                    }
                },
            )

            if (!form.allDay) {
                OutlinedTextField(
                    value = form.startTime.format(DateTimeFormatter.ofPattern("HH:mm")),
                    onValueChange = {},
                    readOnly = true,
                    label = { Text("Start time") },
                    modifier = Modifier.fillMaxWidth().clickable { showStartTimePicker = true },
                    trailingIcon = {
                        IconButton(onClick = { showStartTimePicker = true }) {
                            Icon(Icons.Filled.Schedule, "Pick time")
                        }
                    },
                )
            }

            // End date
            OutlinedTextField(
                value = form.endDate.format(DateTimeFormatter.ISO_LOCAL_DATE),
                onValueChange = {},
                readOnly = true,
                label = { Text("End date") },
                modifier = Modifier.fillMaxWidth().clickable { showEndDatePicker = true },
                trailingIcon = {
                    IconButton(onClick = { showEndDatePicker = true }) {
                        Icon(Icons.Filled.CalendarMonth, "Pick date")
                    }
                },
            )

            if (!form.allDay) {
                OutlinedTextField(
                    value = form.endTime.format(DateTimeFormatter.ofPattern("HH:mm")),
                    onValueChange = {},
                    readOnly = true,
                    label = { Text("End time") },
                    modifier = Modifier.fillMaxWidth().clickable { showEndTimePicker = true },
                    trailingIcon = {
                        IconButton(onClick = { showEndTimePicker = true }) {
                            Icon(Icons.Filled.Schedule, "Pick time")
                        }
                    },
                )
            }

            OutlinedTextField(
                value = form.location,
                onValueChange = onLocationChange,
                label = { Text("Location") },
                modifier = Modifier.fillMaxWidth(),
                singleLine = true,
            )

            OutlinedTextField(
                value = form.description,
                onValueChange = onDescriptionChange,
                label = { Text("Description") },
                modifier = Modifier.fillMaxWidth(),
                minLines = 3,
            )

            if (onDelete != null) {
                OutlinedButton(
                    onClick = { showDeleteDialog = true },
                    colors = ButtonDefaults.outlinedButtonColors(contentColor = MaterialTheme.colorScheme.error),
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    Icon(Icons.Filled.Delete, null)
                    Spacer(Modifier.width(8.dp))
                    Text("Delete Event")
                }
            }
        }
    }

    // Date pickers
    if (showStartDatePicker) {
        val datePickerState = rememberDatePickerState(
            initialSelectedDateMillis = form.startDate.toEpochDay() * 86_400_000L,
        )
        DatePickerDialog(
            onDismissRequest = { showStartDatePicker = false },
            confirmButton = {
                TextButton(onClick = {
                    datePickerState.selectedDateMillis?.let {
                        onStartDateChange(LocalDate.ofEpochDay(it / 86_400_000L))
                    }
                    showStartDatePicker = false
                }) { Text("OK") }
            },
            dismissButton = { TextButton(onClick = { showStartDatePicker = false }) { Text("Cancel") } },
        ) { DatePicker(state = datePickerState) }
    }
    if (showEndDatePicker) {
        val datePickerState = rememberDatePickerState(
            initialSelectedDateMillis = form.endDate.toEpochDay() * 86_400_000L,
        )
        DatePickerDialog(
            onDismissRequest = { showEndDatePicker = false },
            confirmButton = {
                TextButton(onClick = {
                    datePickerState.selectedDateMillis?.let {
                        onEndDateChange(LocalDate.ofEpochDay(it / 86_400_000L))
                    }
                    showEndDatePicker = false
                }) { Text("OK") }
            },
            dismissButton = { TextButton(onClick = { showEndDatePicker = false }) { Text("Cancel") } },
        ) { DatePicker(state = datePickerState) }
    }
    if (showStartTimePicker) {
        val timeState = rememberTimePickerState(form.startTime.hour, form.startTime.minute)
        AlertDialog(
            onDismissRequest = { showStartTimePicker = false },
            title = { Text("Start Time") },
            text = { TimePicker(state = timeState) },
            confirmButton = {
                TextButton(onClick = {
                    onStartTimeChange(LocalTime.of(timeState.hour, timeState.minute))
                    showStartTimePicker = false
                }) { Text("OK") }
            },
            dismissButton = { TextButton(onClick = { showStartTimePicker = false }) { Text("Cancel") } },
        )
    }
    if (showEndTimePicker) {
        val timeState = rememberTimePickerState(form.endTime.hour, form.endTime.minute)
        AlertDialog(
            onDismissRequest = { showEndTimePicker = false },
            title = { Text("End Time") },
            text = { TimePicker(state = timeState) },
            confirmButton = {
                TextButton(onClick = {
                    onEndTimeChange(LocalTime.of(timeState.hour, timeState.minute))
                    showEndTimePicker = false
                }) { Text("OK") }
            },
            dismissButton = { TextButton(onClick = { showEndTimePicker = false }) { Text("Cancel") } },
        )
    }
    if (showDeleteDialog && onDelete != null) {
        AlertDialog(
            onDismissRequest = { showDeleteDialog = false },
            title = { Text("Delete Event?") },
            text = { Text("\"${form.title}\" will be deleted.") },
            confirmButton = {
                TextButton(onClick = { showDeleteDialog = false; onDelete() }) {
                    Text("Delete", color = MaterialTheme.colorScheme.error)
                }
            },
            dismissButton = { TextButton(onClick = { showDeleteDialog = false }) { Text("Cancel") } },
        )
    }
}
