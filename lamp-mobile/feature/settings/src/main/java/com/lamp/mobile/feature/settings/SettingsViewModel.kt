package com.lamp.mobile.feature.settings

import android.content.Context
import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.data.credential.CredentialStore
import com.lamp.mobile.sync.SyncWorker
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject

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
}

@HiltViewModel
class SettingsViewModel @Inject constructor(
    private val credentialStore: CredentialStore,
    @ApplicationContext private val context: Context,
) : MviViewModel<SettingsUiState, SettingsIntent, Nothing>(SettingsUiState()) {

    init {
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
            )
        }
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
        }
    }
}
