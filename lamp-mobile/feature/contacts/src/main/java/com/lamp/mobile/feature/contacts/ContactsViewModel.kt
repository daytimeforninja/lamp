package com.lamp.mobile.feature.contacts

import androidx.lifecycle.viewModelScope
import com.lamp.mobile.core.common.MviViewModel
import com.lamp.mobile.core.data.repository.ContactRepository
import com.lamp.mobile.core.model.Contact
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import java.time.LocalDate
import java.util.UUID
import javax.inject.Inject

data class ContactEditForm(
    val name: String = "",
    val email: String = "",
    val phone: String = "",
    val website: String = "",
    val signal: String = "",
    val preferredMethod: String = "",
    val groups: String = "Personal",
)

data class ContactsUiState(
    val contacts: List<Contact> = emptyList(),
    val inputText: String = "",
    val flippedId: UUID? = null,
    val editingId: UUID? = null,
    val editForm: ContactEditForm = ContactEditForm(),
    val showDeleteConfirm: UUID? = null,
)

sealed class ContactsIntent {
    data class InputChanged(val text: String) : ContactsIntent()
    data object Submit : ContactsIntent()
    data class Flip(val contactId: UUID) : ContactsIntent()
    data class Edit(val contactId: UUID) : ContactsIntent()
    data class Delete(val contactId: UUID) : ContactsIntent()
    data object ConfirmDelete : ContactsIntent()
    data object DismissDelete : ContactsIntent()
    data class MarkContacted(val contactId: UUID) : ContactsIntent()
    data class SetField(val field: String, val value: String) : ContactsIntent()
    data object SaveEdit : ContactsIntent()
    data object CancelEdit : ContactsIntent()
}

@HiltViewModel
class ContactsViewModel @Inject constructor(
    private val contactRepo: ContactRepository,
) : MviViewModel<ContactsUiState, ContactsIntent, Nothing>(ContactsUiState()) {

    init {
        contactRepo.observeAll()
            .onEach { contacts -> updateState { copy(contacts = contacts) } }
            .launchIn(viewModelScope)
    }

    override suspend fun handleIntent(intent: ContactsIntent) {
        when (intent) {
            is ContactsIntent.InputChanged -> updateState { copy(inputText = intent.text) }
            is ContactsIntent.Submit -> {
                val name = currentState.inputText.trim()
                if (name.isNotEmpty()) {
                    contactRepo.save(Contact(name = name))
                    updateState { copy(inputText = "") }
                }
            }
            is ContactsIntent.Flip -> updateState {
                copy(flippedId = if (flippedId == intent.contactId) null else intent.contactId)
            }
            is ContactsIntent.Edit -> {
                val c = contactRepo.getById(intent.contactId) ?: return
                updateState {
                    copy(
                        editingId = intent.contactId,
                        flippedId = null,
                        editForm = ContactEditForm(
                            name = c.name,
                            email = c.email ?: "",
                            phone = c.phone ?: "",
                            website = c.website ?: "",
                            signal = c.signal ?: "",
                            preferredMethod = c.preferredMethod ?: "",
                            groups = c.groups.joinToString(", "),
                        ),
                    )
                }
            }
            is ContactsIntent.SetField -> updateState {
                copy(editForm = when (intent.field) {
                    "name" -> editForm.copy(name = intent.value)
                    "email" -> editForm.copy(email = intent.value)
                    "phone" -> editForm.copy(phone = intent.value)
                    "website" -> editForm.copy(website = intent.value)
                    "signal" -> editForm.copy(signal = intent.value)
                    "preferred" -> editForm.copy(preferredMethod = intent.value)
                    "groups" -> editForm.copy(groups = intent.value)
                    else -> editForm
                })
            }
            is ContactsIntent.SaveEdit -> {
                val id = currentState.editingId ?: return
                val c = contactRepo.getById(id) ?: return
                val form = currentState.editForm
                if (form.name.isBlank()) return
                val groups = form.groups.split(",")
                    .map { it.trim() }
                    .filter { it.isNotEmpty() }
                    .ifEmpty { listOf("Personal") }
                contactRepo.save(c.copy(
                    name = form.name.trim(),
                    email = form.email.ifBlank { null },
                    phone = form.phone.ifBlank { null },
                    website = form.website.ifBlank { null },
                    signal = form.signal.ifBlank { null },
                    preferredMethod = form.preferredMethod.ifBlank { null },
                    groups = groups,
                ))
                updateState { copy(editingId = null) }
            }
            is ContactsIntent.CancelEdit -> updateState { copy(editingId = null) }
            is ContactsIntent.Delete -> updateState { copy(showDeleteConfirm = intent.contactId) }
            is ContactsIntent.ConfirmDelete -> {
                val id = currentState.showDeleteConfirm ?: return
                contactRepo.markDeleted(id)
                updateState { copy(showDeleteConfirm = null, editingId = null, flippedId = null) }
            }
            is ContactsIntent.DismissDelete -> updateState { copy(showDeleteConfirm = null) }
            is ContactsIntent.MarkContacted -> {
                val c = contactRepo.getById(intent.contactId) ?: return
                contactRepo.save(c.copy(lastContacted = LocalDate.now()))
            }
        }
    }
}
