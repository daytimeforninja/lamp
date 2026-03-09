@file:OptIn(ExperimentalMaterial3Api::class)

package com.lamp.mobile.core.common.ui

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.lamp.mobile.core.model.LinkTarget
import java.util.UUID

data class LinkSearchResult(
    val label: String,
    val type: String,
    val target: LinkTarget,
)

@Composable
fun LinkSearchField(
    searchResults: List<LinkSearchResult>,
    onQueryChange: (String) -> Unit,
    onSelect: (LinkTarget) -> Unit,
    modifier: Modifier = Modifier,
) {
    var query by remember { mutableStateOf("") }
    var expanded by remember { mutableStateOf(false) }

    ExposedDropdownMenuBox(
        expanded = expanded && searchResults.isNotEmpty(),
        onExpandedChange = { expanded = it },
        modifier = modifier,
    ) {
        OutlinedTextField(
            value = query,
            onValueChange = {
                query = it
                expanded = true
                onQueryChange(it)
            },
            label = { Text("Search to link...") },
            modifier = Modifier.fillMaxWidth().menuAnchor(),
            singleLine = true,
        )
        ExposedDropdownMenu(
            expanded = expanded && searchResults.isNotEmpty(),
            onDismissRequest = { expanded = false },
        ) {
            searchResults.forEach { result ->
                DropdownMenuItem(
                    text = { Text(result.label) },
                    onClick = {
                        onSelect(result.target)
                        query = ""
                        expanded = false
                    },
                    leadingIcon = {
                        Text(
                            result.type,
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.primary,
                        )
                    },
                )
            }
        }
    }
}
