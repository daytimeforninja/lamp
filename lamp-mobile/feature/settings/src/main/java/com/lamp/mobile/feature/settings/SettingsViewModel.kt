package com.lamp.mobile.feature.settings

import android.content.Context
import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.data.credential.CredentialStore
import com.lamp.mobile.core.data.repository.SyncMetadataRepository
import com.lamp.mobile.core.network.caldav.CalDavClient
import com.lamp.mobile.core.network.carddav.CardDavClient
import com.lamp.mobile.core.network.webdav.WebDavClient
import com.lamp.mobile.sync.SyncWorker
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.launch
import javax.inject.Inject

data class ConnectionTestResult(
    val service: String,
    val success: Boolean,
    val detail: String,
)

data class SettingsUiState(
    val calendarUrl: String = "",
    val calendarUsername: String = "",
    val calendarPassword: String = "",
    val contactsUrl: String = "",
    val contactsUsername: String = "",
    val contactsPassword: String = "",
    val notesUrl: String = "",
    val notesUsername: String = "",
    val notesPassword: String = "",
    val message: String? = null,
    val testingService: String? = null,
    val testResult: ConnectionTestResult? = null,
    val availableContexts: Set<String> = emptySet(),
    val contextInput: String = "",
    val lastSyncTimestamp: Long? = null,
)

sealed class SettingsIntent {
    data class SetCalendarUrl(val url: String) : SettingsIntent()
    data class SetCalendarUsername(val username: String) : SettingsIntent()
    data class SetCalendarPassword(val password: String) : SettingsIntent()
    data class SetContactsUrl(val url: String) : SettingsIntent()
    data class SetContactsUsername(val username: String) : SettingsIntent()
    data class SetContactsPassword(val password: String) : SettingsIntent()
    data class SetNotesUrl(val url: String) : SettingsIntent()
    data class SetNotesUsername(val username: String) : SettingsIntent()
    data class SetNotesPassword(val password: String) : SettingsIntent()
    data object SaveCredentials : SettingsIntent()
    data object SyncNow : SettingsIntent()
    data class TestConnection(val service: String) : SettingsIntent()
    data object DismissTestResult : SettingsIntent()
    data class ContextInputChanged(val text: String) : SettingsIntent()
    data object AddContext : SettingsIntent()
    data class RemoveContext(val context: String) : SettingsIntent()
}

@HiltViewModel
class SettingsViewModel @Inject constructor(
    private val credentialStore: CredentialStore,
    private val syncMetadataRepo: SyncMetadataRepository,
    @ApplicationContext private val context: Context,
) : MviViewModel<SettingsUiState, SettingsIntent, Nothing>(SettingsUiState()) {

    private val prefs = context.getSharedPreferences("lamp_settings", Context.MODE_PRIVATE)

    init {
        val savedContexts = prefs.getStringSet("available_contexts", emptySet()) ?: emptySet()
        updateState {
            copy(
                calendarUrl = credentialStore.getServerUrl("calendars") ?: "",
                calendarUsername = credentialStore.getUsername("calendars") ?: "",
                calendarPassword = credentialStore.getPassword("calendars") ?: "",
                contactsUrl = credentialStore.getServerUrl("contacts") ?: "",
                contactsUsername = credentialStore.getUsername("contacts") ?: "",
                contactsPassword = credentialStore.getPassword("contacts") ?: "",
                notesUrl = credentialStore.getServerUrl("notes") ?: "",
                notesUsername = credentialStore.getUsername("notes") ?: "",
                notesPassword = credentialStore.getPassword("notes") ?: "",
                availableContexts = savedContexts,
            )
        }
        loadLastSyncTimestamp()
    }

    private fun loadLastSyncTimestamp() {
        viewModelScope.launch {
            val metadata = syncMetadataRepo.getAll()
            val latest = metadata.maxByOrNull { it.lastSyncTimestamp }
            updateState { copy(lastSyncTimestamp = latest?.lastSyncTimestamp) }
        }
    }

    private fun saveContexts(contexts: Set<String>) {
        prefs.edit().putStringSet("available_contexts", contexts).apply()
    }

    override suspend fun handleIntent(intent: SettingsIntent) {
        when (intent) {
            is SettingsIntent.SetCalendarUrl -> updateState { copy(calendarUrl = intent.url) }
            is SettingsIntent.SetCalendarUsername -> updateState { copy(calendarUsername = intent.username) }
            is SettingsIntent.SetCalendarPassword -> updateState { copy(calendarPassword = intent.password) }
            is SettingsIntent.SetContactsUrl -> updateState { copy(contactsUrl = intent.url) }
            is SettingsIntent.SetContactsUsername -> updateState { copy(contactsUsername = intent.username) }
            is SettingsIntent.SetContactsPassword -> updateState { copy(contactsPassword = intent.password) }
            is SettingsIntent.SetNotesUrl -> updateState { copy(notesUrl = intent.url) }
            is SettingsIntent.SetNotesUsername -> updateState { copy(notesUsername = intent.username) }
            is SettingsIntent.SetNotesPassword -> updateState { copy(notesPassword = intent.password) }
            is SettingsIntent.SaveCredentials -> {
                val s = currentState
                credentialStore.setServerUrl("calendars", s.calendarUrl)
                credentialStore.setUsername("calendars", s.calendarUsername)
                credentialStore.setPassword("calendars", s.calendarPassword)
                credentialStore.setServerUrl("contacts", s.contactsUrl)
                credentialStore.setUsername("contacts", s.contactsUsername)
                credentialStore.setPassword("contacts", s.contactsPassword)
                credentialStore.setServerUrl("notes", s.notesUrl)
                credentialStore.setUsername("notes", s.notesUsername)
                credentialStore.setPassword("notes", s.notesPassword)
                updateState { copy(message = "Credentials saved") }
            }
            is SettingsIntent.SyncNow -> {
                SyncWorker.enqueueOneTimeSync(context)
                updateState { copy(message = "Sync started") }
            }
            is SettingsIntent.TestConnection -> {
                updateState { copy(testingService = intent.service, testResult = null) }
                val s = currentState
                val result = when (intent.service) {
                    "calendars" -> testCalDav(s.calendarUrl, s.calendarUsername, s.calendarPassword)
                    "contacts" -> testCardDav(s.contactsUrl, s.contactsUsername, s.contactsPassword)
                    "notes" -> testWebDav(s.notesUrl, s.notesUsername, s.notesPassword)
                    else -> ConnectionTestResult(intent.service, false, "Unknown service")
                }
                updateState { copy(testingService = null, testResult = result) }
            }
            is SettingsIntent.DismissTestResult -> {
                updateState { copy(testResult = null) }
            }
            is SettingsIntent.ContextInputChanged -> {
                updateState { copy(contextInput = intent.text) }
            }
            is SettingsIntent.AddContext -> {
                val ctx = currentState.contextInput.trim()
                if (ctx.isNotEmpty()) {
                    val updated = currentState.availableContexts + ctx
                    saveContexts(updated)
                    updateState { copy(availableContexts = updated, contextInput = "") }
                }
            }
            is SettingsIntent.RemoveContext -> {
                val updated = currentState.availableContexts - intent.context
                saveContexts(updated)
                updateState { copy(availableContexts = updated) }
            }
        }
    }

    private suspend fun testCalDav(url: String, user: String, pass: String): ConnectionTestResult {
        if (url.isBlank() || user.isBlank()) return ConnectionTestResult("calendars", false, "URL and username required")
        val client = CalDavClient(url, user, pass)
        return try {
            val calendars = client.discoverCalendars().getOrThrow()
            val taskCals = calendars.filter { it.supportsVtodo }
            val eventCals = calendars.filter { it.supportsVevent }
            ConnectionTestResult("calendars", true,
                "Found ${calendars.size} calendars (${taskCals.size} task, ${eventCals.size} event)")
        } catch (e: Exception) {
            ConnectionTestResult("calendars", false, e.message ?: "Connection failed")
        } finally {
            client.close()
        }
    }

    private suspend fun testCardDav(url: String, user: String, pass: String): ConnectionTestResult {
        if (url.isBlank() || user.isBlank()) return ConnectionTestResult("contacts", false, "URL and username required")
        val client = CardDavClient(url, user, pass)
        return try {
            val contacts = client.fetchContacts().getOrThrow()
            ConnectionTestResult("contacts", true, "Found ${contacts.size} contacts")
        } catch (e: Exception) {
            ConnectionTestResult("contacts", false, e.message ?: "Connection failed")
        } finally {
            client.close()
        }
    }

    private suspend fun testWebDav(url: String, user: String, pass: String): ConnectionTestResult {
        if (url.isBlank() || user.isBlank()) return ConnectionTestResult("notes", false, "URL and username required")
        val client = WebDavClient(url, user, pass)
        return try {
            val files = client.listFiles().getOrThrow()
            ConnectionTestResult("notes", true, "Connected, ${files.size} files found")
        } catch (e: Exception) {
            ConnectionTestResult("notes", false, e.message ?: "Connection failed")
        } finally {
            client.close()
        }
    }
}
