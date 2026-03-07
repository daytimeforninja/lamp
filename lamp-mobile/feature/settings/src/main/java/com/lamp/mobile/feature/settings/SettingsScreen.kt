package com.lamp.mobile.feature.settings

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Sync
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel

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
            url = state.calendarUrl, onUrlChange = { viewModel.onIntent(SettingsIntent.SetCalendarUrl(it)) },
            username = state.calendarUsername, onUsernameChange = { viewModel.onIntent(SettingsIntent.SetCalendarUsername(it)) },
            password = state.calendarPassword, onPasswordChange = { viewModel.onIntent(SettingsIntent.SetCalendarPassword(it)) },
        )

        // CardDAV
        ServiceSection(
            title = "CardDAV (Contacts)",
            url = state.contactsUrl, onUrlChange = { viewModel.onIntent(SettingsIntent.SetContactsUrl(it)) },
            username = state.contactsUsername, onUsernameChange = { viewModel.onIntent(SettingsIntent.SetContactsUsername(it)) },
            password = state.contactsPassword, onPasswordChange = { viewModel.onIntent(SettingsIntent.SetContactsPassword(it)) },
        )

        // WebDAV
        ServiceSection(
            title = "WebDAV (Notes)",
            url = state.notesUrl, onUrlChange = { viewModel.onIntent(SettingsIntent.SetNotesUrl(it)) },
            username = state.notesUsername, onUsernameChange = { viewModel.onIntent(SettingsIntent.SetNotesUsername(it)) },
            password = state.notesPassword, onPasswordChange = { viewModel.onIntent(SettingsIntent.SetNotesPassword(it)) },
        )

        Button(
            onClick = { viewModel.onIntent(SettingsIntent.SaveCredentials) },
            modifier = Modifier.fillMaxWidth().padding(16.dp),
        ) {
            Text("Save Credentials")
        }

        Spacer(Modifier.height(32.dp))
    }
}

@Composable
private fun ServiceSection(
    title: String,
    url: String, onUrlChange: (String) -> Unit,
    username: String, onUsernameChange: (String) -> Unit,
    password: String, onPasswordChange: (String) -> Unit,
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
    }
}
