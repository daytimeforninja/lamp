package com.lamp.mobile.core.data.repository

import com.lamp.mobile.core.database.converter.EntityMappers.toDomain
import com.lamp.mobile.core.database.converter.EntityMappers.toEntity
import com.lamp.mobile.core.database.dao.ContactDao
import com.lamp.mobile.core.model.Contact
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import java.util.UUID
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class ContactRepositoryImpl @Inject constructor(
    private val contactDao: ContactDao,
) : ContactRepository {

    override fun observeAll(): Flow<List<Contact>> =
        contactDao.observeAll().map { list -> list.map { it.toDomain() } }

    override suspend fun getById(id: UUID): Contact? =
        contactDao.getById(id.toString())?.toDomain()

    override suspend fun save(contact: Contact, markDirty: Boolean) =
        contactDao.upsert(contact.toEntity(syncDirty = markDirty))

    override suspend fun saveAll(contacts: List<Contact>) =
        contactDao.upsertAll(contacts.map { it.toEntity() })

    override suspend fun delete(id: UUID) =
        contactDao.deleteById(id.toString())

    override suspend fun markDeleted(id: UUID) =
        contactDao.markDeleted(id.toString())

    override suspend fun getByHref(href: String): Contact? =
        contactDao.getByHref(href)?.toDomain()

    override suspend fun getAll(): List<Contact> =
        contactDao.getAll().map { it.toDomain() }

    override suspend fun getDirty(): List<Contact> =
        contactDao.getDirty().map { it.toDomain() }

    override suspend fun getDeleted(): List<Contact> =
        contactDao.getDeleted().map { it.toDomain() }

    override suspend fun markSynced(id: UUID, syncHref: String, syncEtag: String?) =
        contactDao.markSynced(id.toString(), syncHref, syncEtag)
}
