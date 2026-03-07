package com.lamp.mobile.core.common.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.outlined.Circle
import androidx.compose.material.icons.outlined.ExpandMore
import androidx.compose.material.icons.outlined.ExpandLess
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.lamp.mobile.core.model.Priority
import com.lamp.mobile.core.model.Task
import com.lamp.mobile.core.model.TaskState

@Composable
fun TaskRow(
    task: Task,
    onToggleDone: () -> Unit,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    showProject: Boolean = true,
    trailing: @Composable (() -> Unit)? = null,
) {
    var expanded by remember { mutableStateOf(false) }

    Column(modifier = modifier.fillMaxWidth()) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .clickable(onClick = onClick)
                .padding(horizontal = 16.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            IconButton(onClick = onToggleDone, modifier = Modifier.size(24.dp)) {
                Icon(
                    imageVector = if (task.state.isDone) Icons.Filled.CheckCircle else Icons.Outlined.Circle,
                    contentDescription = if (task.state.isDone) "Done" else "Mark done",
                    tint = if (task.state.isDone) MaterialTheme.colorScheme.primary
                    else MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }

            Spacer(Modifier.width(12.dp))

            Column(modifier = Modifier.weight(1f)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    task.priority?.let { PriorityBadge(it) }
                    if (task.priority != null) Spacer(Modifier.width(6.dp))
                    Text(
                        text = task.title,
                        style = MaterialTheme.typography.bodyLarge,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        textDecoration = if (task.state.isDone) TextDecoration.LineThrough else null,
                    )
                }

                Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                    if (showProject && task.project != null) {
                        AssistChip(
                            onClick = {},
                            label = { Text(task.project!!, style = MaterialTheme.typography.labelSmall) },
                            modifier = Modifier.height(20.dp),
                        )
                    }
                    task.esc?.let { EscBadge(it) }
                    if (task.contexts.isNotEmpty()) {
                        ContextChipRow(contexts = task.contexts, compact = true)
                    }
                }
            }

            trailing?.invoke()

            if (task.notes.isNotBlank()) {
                IconButton(
                    onClick = { expanded = !expanded },
                    modifier = Modifier.size(24.dp),
                ) {
                    Icon(
                        if (expanded) Icons.Outlined.ExpandLess else Icons.Outlined.ExpandMore,
                        contentDescription = "Toggle notes",
                    )
                }
            }
        }

        if (expanded && task.notes.isNotBlank()) {
            Text(
                text = task.notes,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(start = 52.dp, end = 16.dp, bottom = 8.dp),
            )
        }

        HorizontalDivider(modifier = Modifier.padding(start = 52.dp))
    }
}
