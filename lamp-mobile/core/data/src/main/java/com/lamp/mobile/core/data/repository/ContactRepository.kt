package com.lamp.mobile.core.data.repository

import com.lamp.mobile.core.model.Contact
import kotlinx.coroutines.flow.Flow
import java.util.UUID

interface ContactRepository {
    fun observeAll(): Flow<List<Contact>>
    suspend fun getById(id: UUID): Contact?
    suspend fun save(contact: Contact, markDirty: Boolean = true)
    suspend fun saveAll(contacts: List<Contact>)
    suspend fun delete(id: UUID)
    suspend fun markDeleted(id: UUID)
    suspend fun getByHref(href: String): Contact?
    suspend fun getAll(): List<Contact>
    suspend fun getDirty(): List<Contact>
    suspend fun getDeleted(): List<Contact>
    suspend fun markSynced(id: UUID, syncHref: String, syncEtag: String?)
}
