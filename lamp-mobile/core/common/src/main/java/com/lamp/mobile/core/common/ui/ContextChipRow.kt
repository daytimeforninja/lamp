package com.lamp.mobile.core.common.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.height
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun ContextChipRow(
    contexts: List<String>,
    modifier: Modifier = Modifier,
    compact: Boolean = false,
    onContextClick: ((String) -> Unit)? = null,
) {
    FlowRow(
        modifier = modifier,
        horizontalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        for (ctx in contexts) {
            if (compact) {
                Text(
                    text = ctx,
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.primary,
                )
            } else {
                FilterChip(
                    selected = false,
                    onClick = { onContextClick?.invoke(ctx) },
                    label = { Text(ctx, style = MaterialTheme.typography.labelSmall) },
                    modifier = Modifier.height(28.dp),
                )
            }
        }
    }
}
