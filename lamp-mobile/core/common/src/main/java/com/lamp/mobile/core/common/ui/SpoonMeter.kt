package com.lamp.mobile.core.common.ui

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun SpoonMeter(
    spent: Int,
    budget: Int,
    modifier: Modifier = Modifier,
) {
    val progress = if (budget > 0) (spent.toFloat() / budget).coerceIn(0f, 1f) else 0f
    val remaining = (budget - spent).coerceAtLeast(0)

    Column(modifier = modifier) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                text = "$remaining sp remaining",
                style = MaterialTheme.typography.labelMedium,
            )
            Text(
                text = "$spent / $budget",
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        Spacer(Modifier.height(4.dp))
        LinearProgressIndicator(
            progress = { progress },
            modifier = Modifier.fillMaxWidth().height(8.dp),
            color = if (progress > 0.9f) MaterialTheme.colorScheme.error
            else MaterialTheme.colorScheme.primary,
            trackColor = MaterialTheme.colorScheme.surfaceVariant,
        )
    }
}
