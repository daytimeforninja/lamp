package com.lamp.mobile.feature.calendar

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ChevronLeft
import androidx.compose.material.icons.filled.ChevronRight
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import java.time.DayOfWeek
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import java.time.format.TextStyle
import java.util.Locale

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CalendarScreen(
    viewModel: CalendarViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()
    val month = state.currentMonth
    val firstDay = month.atDay(1)
    val daysInMonth = month.lengthOfMonth()
    val startDow = firstDay.dayOfWeek.value % 7 // 0=Sun
    val eventDates = state.events.map { it.start.toLocalDate() }.toSet()
    val selectedDayEvents = state.events.filter { it.start.toLocalDate() == state.selectedDate }

    Column(modifier = Modifier.fillMaxSize()) {
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
                )
                HorizontalDivider()
            }
        }
    }
}
