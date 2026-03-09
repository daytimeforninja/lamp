@file:OptIn(ExperimentalMaterial3Api::class)

package com.lamp.mobile.feature.accounts

import android.content.Intent
import android.net.Uri
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.OpenInBrowser
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.common.ui.QuickCaptureBar
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import java.time.temporal.ChronoUnit

@Composable
fun AccountsScreen(
    viewModel: AccountsViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()
    val context = LocalContext.current

    Column(modifier = Modifier.fillMaxSize()) {
        TopAppBar(title = { Text("Accounts") })

        QuickCaptureBar(
            value = state.inputText,
            onValueChange = { viewModel.onIntent(AccountsIntent.InputChanged(it)) },
            onSubmit = { viewModel.onIntent(AccountsIntent.Submit) },
            placeholder = "New account...",
        )

        LazyColumn(modifier = Modifier.fillMaxSize()) {
            items(state.accounts, key = { it.id }) { account ->
                val isExpanded = state.expandedId == account.id
                Card(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 16.dp, vertical = 4.dp)
                        .clickable { viewModel.onIntent(AccountsIntent.ToggleExpanded(account.id)) },
                ) {
                    if (!isExpanded) {
                        // Collapsed view
                        Row(
                            modifier = Modifier.padding(16.dp).fillMaxWidth(),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Column(modifier = Modifier.weight(1f)) {
                                Text(account.name, style = MaterialTheme.typography.bodyLarge)
                                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                                    if (account.url.isNotBlank()) {
                                        AssistChip(onClick = {}, label = {
                                            Text(account.url.take(30), style = MaterialTheme.typography.labelSmall)
                                        })
                                    }
                                    val lastCheckedText = account.lastChecked?.let {
                                        val days = ChronoUnit.DAYS.between(it, LocalDate.now())
                                        if (days == 0L) "Today" else "$days days ago"
                                    } ?: "Never checked"
                                    Text(lastCheckedText, style = MaterialTheme.typography.labelSmall,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant)
                                }
                            }
                            IconButton(onClick = { viewModel.onIntent(AccountsIntent.Delete(account.id)) }) {
                                Icon(Icons.Filled.Delete, "Delete", tint = MaterialTheme.colorScheme.error)
                            }
                        }
                    } else {
                        // Expanded view
                        Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                            OutlinedTextField(
                                value = account.name,
                                onValueChange = { viewModel.onIntent(AccountsIntent.UpdateName(account.id, it)) },
                                label = { Text("Name") },
                                modifier = Modifier.fillMaxWidth(),
                                singleLine = true,
                            )
                            OutlinedTextField(
                                value = account.url,
                                onValueChange = { viewModel.onIntent(AccountsIntent.UpdateUrl(account.id, it)) },
                                label = { Text("URL") },
                                modifier = Modifier.fillMaxWidth(),
                                singleLine = true,
                            )
                            OutlinedTextField(
                                value = account.notes,
                                onValueChange = { viewModel.onIntent(AccountsIntent.UpdateNotes(account.id, it)) },
                                label = { Text("Notes") },
                                modifier = Modifier.fillMaxWidth(),
                                minLines = 2,
                            )
                            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                                FilledTonalButton(onClick = {
                                    viewModel.onIntent(AccountsIntent.MarkChecked(account.id))
                                }) { Text("Mark Checked") }
                                if (account.url.isNotBlank()) {
                                    OutlinedButton(onClick = {
                                        val url = if (account.url.startsWith("http")) account.url else "https://${account.url}"
                                        context.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url)))
                                    }) {
                                        Icon(Icons.Filled.OpenInBrowser, null, modifier = Modifier.size(16.dp))
                                        Spacer(Modifier.width(4.dp))
                                        Text("Open URL")
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
