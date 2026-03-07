package com.lamp.mobile.feature.review

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.outlined.Circle
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

private val REVIEW_STEPS = listOf(
    "Clear inbox to zero",
    "Review Next Actions list — still relevant?",
    "Review Waiting For — follow up needed?",
    "Review Projects — each has a next action?",
    "Review Someday/Maybe — activate anything?",
    "Review calendar (past week) — any follow-ups?",
    "Review calendar (upcoming) — any prep needed?",
    "Review Habits — streaks intact?",
    "Process notes and ideas",
    "Review goals and vision",
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ReviewScreen() {
    var completed by remember { mutableStateOf(setOf<Int>()) }

    Column(modifier = Modifier.fillMaxSize()) {
        TopAppBar(title = { Text("Weekly Review") })

        LinearProgressIndicator(
            progress = { completed.size.toFloat() / REVIEW_STEPS.size },
            modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp),
        )

        Text(
            "${completed.size} / ${REVIEW_STEPS.size} complete",
            style = MaterialTheme.typography.labelMedium,
            modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp),
        )

        LazyColumn(modifier = Modifier.fillMaxSize()) {
            itemsIndexed(REVIEW_STEPS) { index, step ->
                val isDone = index in completed
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 16.dp, vertical = 12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    IconButton(
                        onClick = {
                            completed = if (isDone) completed - index else completed + index
                        },
                        modifier = Modifier.size(24.dp),
                    ) {
                        Icon(
                            if (isDone) Icons.Filled.CheckCircle else Icons.Outlined.Circle,
                            "Toggle",
                            tint = if (isDone) MaterialTheme.colorScheme.primary
                            else MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    Spacer(Modifier.width(12.dp))
                    Text(step, style = MaterialTheme.typography.bodyMedium)
                }
                HorizontalDivider(modifier = Modifier.padding(start = 52.dp))
            }
        }
    }
}
