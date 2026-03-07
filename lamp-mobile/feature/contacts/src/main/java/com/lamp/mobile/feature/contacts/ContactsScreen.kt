package com.lamp.mobile.feature.contacts

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.Person
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.lamp.mobile.core.common.ui.QuickCaptureBar
import com.lamp.mobile.core.model.ContactCategory
import java.time.format.DateTimeFormatter

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ContactsScreen(
    viewModel: ContactsViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()
    val personalContacts = state.contacts.filter { it.category == ContactCategory.PERSONAL }
    val serviceContacts = state.contacts.filter { it.category == ContactCategory.SERVICE }

    // Delete confirmation dialog
    if (state.showDeleteConfirm != null) {
        val contact = state.contacts.find { it.id == state.showDeleteConfirm }
        AlertDialog(
            onDismissRequest = { viewModel.onIntent(ContactsIntent.DismissDelete) },
            title = { Text("Delete Contact") },
            text = { Text("Delete ${contact?.name ?: "this contact"}?") },
            confirmButton = {
                TextButton(onClick = { viewModel.onIntent(ContactsIntent.ConfirmDelete) }) {
                    Text("Delete")
                }
            },
            dismissButton = {
                TextButton(onClick = { viewModel.onIntent(ContactsIntent.DismissDelete) }) {
                    Text("Cancel")
                }
            },
        )
    }

    Column(modifier = Modifier.fillMaxSize()) {
        TopAppBar(title = { Text("Contacts") })

        // Edit form
        if (state.editingId != null) {
            ContactEditForm(
                form = state.editForm,
                onFieldChange = { field, value ->
                    viewModel.onIntent(ContactsIntent.SetField(field, value))
                },
                onSave = { viewModel.onIntent(ContactsIntent.SaveEdit) },
                onCancel = { viewModel.onIntent(ContactsIntent.CancelEdit) },
                onDelete = { viewModel.onIntent(ContactsIntent.Delete(state.editingId!!)) },
            )
        } else {
            QuickCaptureBar(
                value = state.inputText,
                onValueChange = { viewModel.onIntent(ContactsIntent.InputChanged(it)) },
                onSubmit = { viewModel.onIntent(ContactsIntent.Submit) },
                placeholder = "Add contact...",
            )
        }

        LazyColumn(modifier = Modifier.fillMaxSize()) {
            if (personalContacts.isNotEmpty()) {
                item {
                    Text("Personal", style = MaterialTheme.typography.titleSmall,
                        modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp))
                }
                items(personalContacts, key = { it.id }) { contact ->
                    ContactCard(
                        contact = contact,
                        isFlipped = contact.id == state.flippedId,
                        onFlip = { viewModel.onIntent(ContactsIntent.Flip(contact.id)) },
                        onMarkContacted = { viewModel.onIntent(ContactsIntent.MarkContacted(contact.id)) },
                        onEdit = { viewModel.onIntent(ContactsIntent.Edit(contact.id)) },
                        onDelete = { viewModel.onIntent(ContactsIntent.Delete(contact.id)) },
                    )
                }
            }

            if (serviceContacts.isNotEmpty()) {
                item {
                    Text("Service", style = MaterialTheme.typography.titleSmall,
                        modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp))
                }
                items(serviceContacts, key = { it.id }) { contact ->
                    ContactCard(
                        contact = contact,
                        isFlipped = contact.id == state.flippedId,
                        onFlip = { viewModel.onIntent(ContactsIntent.Flip(contact.id)) },
                        onMarkContacted = { viewModel.onIntent(ContactsIntent.MarkContacted(contact.id)) },
                        onEdit = { viewModel.onIntent(ContactsIntent.Edit(contact.id)) },
                        onDelete = { viewModel.onIntent(ContactsIntent.Delete(contact.id)) },
                    )
                }
            }
        }
    }
}

@Composable
private fun ContactEditForm(
    form: ContactEditForm,
    onFieldChange: (String, String) -> Unit,
    onSave: () -> Unit,
    onCancel: () -> Unit,
    onDelete: () -> Unit,
) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 8.dp),
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            OutlinedTextField(
                value = form.name,
                onValueChange = { onFieldChange("name", it) },
                label = { Text("Name") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            Spacer(Modifier.height(8.dp))
            OutlinedTextField(
                value = form.email,
                onValueChange = { onFieldChange("email", it) },
                label = { Text("Email") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            Spacer(Modifier.height(8.dp))
            OutlinedTextField(
                value = form.phone,
                onValueChange = { onFieldChange("phone", it) },
                label = { Text("Phone") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            Spacer(Modifier.height(8.dp))
            OutlinedTextField(
                value = form.website,
                onValueChange = { onFieldChange("website", it) },
                label = { Text("Website") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            Spacer(Modifier.height(8.dp))
            OutlinedTextField(
                value = form.signal,
                onValueChange = { onFieldChange("signal", it) },
                label = { Text("Signal") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            Spacer(Modifier.height(8.dp))
            OutlinedTextField(
                value = form.preferredMethod,
                onValueChange = { onFieldChange("preferred", it) },
                label = { Text("Preferred Method") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            Spacer(Modifier.height(8.dp))
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text("Category:", style = MaterialTheme.typography.bodyMedium)
                Spacer(Modifier.width(8.dp))
                FilterChip(
                    selected = form.category == ContactCategory.PERSONAL,
                    onClick = { onFieldChange("category", "Personal") },
                    label = { Text("Personal") },
                )
                Spacer(Modifier.width(8.dp))
                FilterChip(
                    selected = form.category == ContactCategory.SERVICE,
                    onClick = { onFieldChange("category", "Service") },
                    label = { Text("Service") },
                )
            }
            Spacer(Modifier.height(12.dp))
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                TextButton(onClick = onDelete, colors = ButtonDefaults.textButtonColors(
                    contentColor = MaterialTheme.colorScheme.error,
                )) {
                    Text("Delete")
                }
                Row {
                    OutlinedButton(onClick = onCancel) { Text("Cancel") }
                    Spacer(Modifier.width(8.dp))
                    Button(onClick = onSave) { Text("Save") }
                }
            }
        }
    }
}

@Composable
private fun ContactCard(
    contact: com.lamp.mobile.core.model.Contact,
    isFlipped: Boolean,
    onFlip: () -> Unit,
    onMarkContacted: () -> Unit,
    onEdit: () -> Unit,
    onDelete: () -> Unit,
) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 4.dp),
    ) {
        if (!isFlipped) {
            // Front: name, category, last contacted — tap to flip
            Row(
                modifier = Modifier.clickable(onClick = onFlip).padding(16.dp).fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Icon(Icons.Filled.Person, "Contact", modifier = Modifier.size(40.dp))
                Spacer(Modifier.width(12.dp))
                Column(modifier = Modifier.weight(1f)) {
                    Text(contact.name, style = MaterialTheme.typography.bodyLarge)
                    contact.lastContacted?.let {
                        Text("Last: ${it.format(DateTimeFormatter.ISO_LOCAL_DATE)}",
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
                contact.preferredMethod?.let {
                    AssistChip(onClick = {}, label = { Text(it) })
                }
            }
        } else {
            // Back: all fields + actions — tap text area to flip back
            Column(modifier = Modifier.padding(16.dp)) {
                Column(modifier = Modifier.clickable(onClick = onFlip)) {
                Text(contact.name, style = MaterialTheme.typography.titleMedium)
                Spacer(Modifier.height(8.dp))
                contact.email?.let { Text("Email: $it", style = MaterialTheme.typography.bodySmall) }
                contact.phone?.let { Text("Phone: $it", style = MaterialTheme.typography.bodySmall) }
                contact.website?.let { Text("Web: $it", style = MaterialTheme.typography.bodySmall) }
                contact.signal?.let { Text("Signal: $it", style = MaterialTheme.typography.bodySmall) }
                }
                Spacer(Modifier.height(8.dp))
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                ) {
                    FilledTonalButton(onClick = onMarkContacted) {
                        Text("Mark Contacted")
                    }
                    Row {
                        IconButton(onClick = onEdit) {
                            Icon(Icons.Filled.Edit, "Edit")
                        }
                        IconButton(onClick = onDelete) {
                            Icon(Icons.Filled.Delete, "Delete",
                                tint = MaterialTheme.colorScheme.error)
                        }
                    }
                }
            }
        }
    }
}
