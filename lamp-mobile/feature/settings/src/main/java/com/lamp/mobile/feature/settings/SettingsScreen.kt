@file:OptIn(ExperimentalLayoutApi::class)

package com.lamp.mobile.feature.settings

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Error
import androidx.compose.material.icons.filled.Sync
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(
    viewModel: SettingsViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()

    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState()),
    ) {
        TopAppBar(
            title = { Text("Settings") },
            actions = {
                IconButton(onClick = { viewModel.onIntent(SettingsIntent.SyncNow) }) {
                    Icon(Icons.Filled.Sync, "Sync Now")
                }
            },
        )

        state.message?.let {
            Snackbar(modifier = Modifier.padding(16.dp)) { Text(it) }
        }

        // CalDAV
        ServiceSection(
            title = "CalDAV (Tasks & Events)",
            serviceKey = "calendars",
            url = state.calendarUrl, onUrlChange = { viewModel.onIntent(SettingsIntent.SetCalendarUrl(it)) },
            username = state.calendarUsername, onUsernameChange = { viewModel.onIntent(SettingsIntent.SetCalendarUsername(it)) },
            password = state.calendarPassword, onPasswordChange = { viewModel.onIntent(SettingsIntent.SetCalendarPassword(it)) },
            onTest = { viewModel.onIntent(SettingsIntent.TestConnection("calendars")) },
            isTesting = state.testingService == "calendars",
            testResult = state.testResult?.takeIf { it.service == "calendars" },
        )

        // CardDAV
        ServiceSection(
            title = "CardDAV (Contacts)",
            serviceKey = "contacts",
            url = state.contactsUrl, onUrlChange = { viewModel.onIntent(SettingsIntent.SetContactsUrl(it)) },
            username = state.contactsUsername, onUsernameChange = { viewModel.onIntent(SettingsIntent.SetContactsUsername(it)) },
            password = state.contactsPassword, onPasswordChange = { viewModel.onIntent(SettingsIntent.SetContactsPassword(it)) },
            onTest = { viewModel.onIntent(SettingsIntent.TestConnection("contacts")) },
            isTesting = state.testingService == "contacts",
            testResult = state.testResult?.takeIf { it.service == "contacts" },
        )

        // WebDAV
        ServiceSection(
            title = "WebDAV (Notes)",
            serviceKey = "notes",
            url = state.notesUrl, onUrlChange = { viewModel.onIntent(SettingsIntent.SetNotesUrl(it)) },
            username = state.notesUsername, onUsernameChange = { viewModel.onIntent(SettingsIntent.SetNotesUsername(it)) },
            password = state.notesPassword, onPasswordChange = { viewModel.onIntent(SettingsIntent.SetNotesPassword(it)) },
            onTest = { viewModel.onIntent(SettingsIntent.TestConnection("notes")) },
            isTesting = state.testingService == "notes",
            testResult = state.testResult?.takeIf { it.service == "notes" },
        )

        // IMAP
        ImapServiceSection(
            state = state,
            onHostChange = { viewModel.onIntent(SettingsIntent.SetImapHost(it)) },
            onUsernameChange = { viewModel.onIntent(SettingsIntent.SetImapUsername(it)) },
            onPasswordChange = { viewModel.onIntent(SettingsIntent.SetImapPassword(it)) },
            onFolderChange = { viewModel.onIntent(SettingsIntent.SetImapFolder(it)) },
            onTest = { viewModel.onIntent(SettingsIntent.TestConnection("imap")) },
            isTesting = state.testingService == "imap",
            testResult = state.testResult?.takeIf { it.service == "imap" },
        )

        Button(
            onClick = { viewModel.onIntent(SettingsIntent.SaveCredentials) },
            modifier = Modifier.fillMaxWidth().padding(16.dp),
        ) {
            Text("Save Credentials")
        }

        HorizontalDivider(modifier = Modifier.padding(vertical = 8.dp))

        // Available Contexts
        Column(modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp)) {
            Text("Available Contexts", style = MaterialTheme.typography.titleSmall)
            Spacer(Modifier.height(8.dp))
            FlowRow(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                state.availableContexts.sorted().forEach { ctx ->
                    InputChip(
                        selected = false,
                        onClick = { viewModel.onIntent(SettingsIntent.RemoveContext(ctx)) },
                        label = { Text(ctx) },
                        trailingIcon = { Icon(Icons.Filled.Close, "Remove", modifier = Modifier.size(16.dp)) },
                    )
                }
            }
            Spacer(Modifier.height(8.dp))
            Row(verticalAlignment = Alignment.CenterVertically) {
                OutlinedTextField(
                    value = state.contextInput,
                    onValueChange = { viewModel.onIntent(SettingsIntent.ContextInputChanged(it)) },
                    label = { Text("New context") },
                    modifier = Modifier.weight(1f),
                    singleLine = true,
                )
                Spacer(Modifier.width(8.dp))
                IconButton(
                    onClick = { viewModel.onIntent(SettingsIntent.AddContext) },
                    enabled = state.contextInput.isNotBlank(),
                ) {
                    Icon(Icons.Filled.Add, "Add")
                }
            }
        }

        HorizontalDivider(modifier = Modifier.padding(vertical = 8.dp))

        // Last sync timestamp
        Column(modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp)) {
            Text("Sync Status", style = MaterialTheme.typography.titleSmall)
            Spacer(Modifier.height(4.dp))
            val ts = state.lastSyncTimestamp
            val syncText = if (ts != null) {
                val fmt = SimpleDateFormat("MMM d, yyyy HH:mm", Locale.getDefault())
                "Last sync: ${fmt.format(Date(ts))}"
            } else {
                "Never synced"
            }
            Text(syncText, style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant)
        }

        Spacer(Modifier.height(32.dp))
    }
}

@Composable
private fun ImapServiceSection(
    state: SettingsUiState,
    onHostChange: (String) -> Unit,
    onUsernameChange: (String) -> Unit,
    onPasswordChange: (String) -> Unit,
    onFolderChange: (String) -> Unit,
    onTest: () -> Unit,
    isTesting: Boolean,
    testResult: ConnectionTestResult?,
) {
    Column(modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp)) {
        Text("IMAP (Email Inbox)", style = MaterialTheme.typography.titleSmall)
        Spacer(Modifier.height(8.dp))
        OutlinedTextField(
            value = state.imapHost, onValueChange = onHostChange,
            label = { Text("IMAP Host") },
            modifier = Modifier.fillMaxWidth(), singleLine = true,
        )
        Spacer(Modifier.height(4.dp))
        OutlinedTextField(
            value = state.imapUsername, onValueChange = onUsernameChange,
            label = { Text("Username") },
            modifier = Modifier.fillMaxWidth(), singleLine = true,
        )
        Spacer(Modifier.height(4.dp))
        OutlinedTextField(
            value = state.imapPassword, onValueChange = onPasswordChange,
            label = { Text("Password") },
            modifier = Modifier.fillMaxWidth(), singleLine = true,
            visualTransformation = PasswordVisualTransformation(),
        )
        Spacer(Modifier.height(4.dp))
        OutlinedTextField(
            value = state.imapFolder, onValueChange = onFolderChange,
            label = { Text("Folder (e.g. INBOX.flup)") },
            modifier = Modifier.fillMaxWidth(), singleLine = true,
        )
        Spacer(Modifier.height(8.dp))
        OutlinedButton(
            onClick = onTest,
            enabled = !isTesting && state.imapHost.isNotBlank(),
            modifier = Modifier.align(Alignment.End),
        ) {
            if (isTesting) {
                CircularProgressIndicator(modifier = Modifier.size(16.dp), strokeWidth = 2.dp)
                Spacer(Modifier.width(8.dp))
                Text("Testing...")
            } else {
                Text("Test Connection")
            }
        }
        if (testResult != null) {
            Spacer(Modifier.height(8.dp))
            val containerColor = if (testResult.success) MaterialTheme.colorScheme.primaryContainer
                else MaterialTheme.colorScheme.errorContainer
            val contentColor = if (testResult.success) MaterialTheme.colorScheme.onPrimaryContainer
                else MaterialTheme.colorScheme.onErrorContainer
            Card(colors = CardDefaults.cardColors(containerColor = containerColor, contentColor = contentColor),
                modifier = Modifier.fillMaxWidth()) {
                Row(modifier = Modifier.padding(12.dp), verticalAlignment = Alignment.CenterVertically) {
                    Icon(if (testResult.success) Icons.Filled.CheckCircle else Icons.Filled.Error, null, Modifier.size(24.dp))
                    Spacer(Modifier.width(12.dp))
                    Text(testResult.detail, style = MaterialTheme.typography.bodyMedium)
                }
            }
        }
    }
}

@Composable
private fun ServiceSection(
    title: String,
    serviceKey: String,
    url: String, onUrlChange: (String) -> Unit,
    username: String, onUsernameChange: (String) -> Unit,
    password: String, onPasswordChange: (String) -> Unit,
    onTest: () -> Unit,
    isTesting: Boolean,
    testResult: ConnectionTestResult?,
) {
    Column(modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp)) {
        Text(title, style = MaterialTheme.typography.titleSmall)
        Spacer(Modifier.height(8.dp))
        OutlinedTextField(
            value = url, onValueChange = onUrlChange,
            label = { Text("Server URL") },
            modifier = Modifier.fillMaxWidth(), singleLine = true,
        )
        Spacer(Modifier.height(4.dp))
        OutlinedTextField(
            value = username, onValueChange = onUsernameChange,
            label = { Text("Username") },
            modifier = Modifier.fillMaxWidth(), singleLine = true,
        )
        Spacer(Modifier.height(4.dp))
        OutlinedTextField(
            value = password, onValueChange = onPasswordChange,
            label = { Text("Password") },
            modifier = Modifier.fillMaxWidth(), singleLine = true,
            visualTransformation = PasswordVisualTransformation(),
        )
        Spacer(Modifier.height(8.dp))
        OutlinedButton(
            onClick = onTest,
            enabled = !isTesting && url.isNotBlank(),
            modifier = Modifier.align(Alignment.End),
        ) {
            if (isTesting) {
                CircularProgressIndicator(modifier = Modifier.size(16.dp), strokeWidth = 2.dp)
                Spacer(Modifier.width(8.dp))
                Text("Testing...")
            } else {
                Text("Test Connection")
            }
        }
        if (testResult != null) {
            Spacer(Modifier.height(8.dp))
            val containerColor = if (testResult.success)
                MaterialTheme.colorScheme.primaryContainer
            else
                MaterialTheme.colorScheme.errorContainer
            val contentColor = if (testResult.success)
                MaterialTheme.colorScheme.onPrimaryContainer
            else
                MaterialTheme.colorScheme.onErrorContainer
            Card(
                colors = CardDefaults.cardColors(
                    containerColor = containerColor,
                    contentColor = contentColor,
                ),
                modifier = Modifier.fillMaxWidth(),
            ) {
                Row(
                    modifier = Modifier.padding(12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Icon(
                        if (testResult.success) Icons.Filled.CheckCircle else Icons.Filled.Error,
                        contentDescription = null,
                        modifier = Modifier.size(24.dp),
                    )
                    Spacer(Modifier.width(12.dp))
                    Text(
                        text = testResult.detail,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
            }
        }
    }
}
