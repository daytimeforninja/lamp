package com.lamp.mobile.feature.lists

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.outlined.Circle
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.common.ui.QuickCaptureBar
import com.lamp.mobile.core.model.ListKind

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ListsScreen(
    isMedia: Boolean = true,
    viewModel: ListsViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()

    LaunchedEffect(isMedia) {
        viewModel.onIntent(ListsIntent.SetKind(if (isMedia) ListKind.MEDIA else ListKind.SHOPPING))
    }

    Column(modifier = Modifier.fillMaxSize()) {
        TopAppBar(title = { Text(if (isMedia) "Media" else "Shopping") })

        QuickCaptureBar(
            value = state.inputText,
            onValueChange = { viewModel.onIntent(ListsIntent.InputChanged(it)) },
            onSubmit = { viewModel.onIntent(ListsIntent.Submit) },
            placeholder = if (isMedia) "Add media..." else "Add item...",
        )

        LazyColumn(modifier = Modifier.fillMaxSize()) {
            items(state.items, key = { it.id }) { item ->
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 16.dp, vertical = 12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    IconButton(
                        onClick = { viewModel.onIntent(ListsIntent.ToggleDone(item.id)) },
                        modifier = Modifier.size(24.dp),
                    ) {
                        Icon(
                            if (item.done) Icons.Filled.CheckCircle else Icons.Outlined.Circle,
                            contentDescription = "Toggle",
                            tint = if (item.done) MaterialTheme.colorScheme.primary
                            else MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    Spacer(Modifier.width(12.dp))
                    Text(
                        item.title,
                        modifier = Modifier.weight(1f),
                        textDecoration = if (item.done) TextDecoration.LineThrough else null,
                    )
                    IconButton(onClick = { viewModel.onIntent(ListsIntent.Delete(item.id)) }) {
                        Icon(Icons.Filled.Delete, "Delete",
                            tint = MaterialTheme.colorScheme.error)
                    }
                }
                HorizontalDivider(modifier = Modifier.padding(start = 52.dp))
            }
        }
    }
}
