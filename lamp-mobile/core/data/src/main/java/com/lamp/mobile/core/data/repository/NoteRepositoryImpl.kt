package com.lamp.mobile.core.data.repository

import com.lamp.mobile.core.database.converter.EntityMappers.toDomain
import com.lamp.mobile.core.database.converter.EntityMappers.toEntity
import com.lamp.mobile.core.database.dao.NoteDao
import com.lamp.mobile.core.model.Note
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import java.util.UUID
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class NoteRepositoryImpl @Inject constructor(
    private val noteDao: NoteDao,
) : NoteRepository {

    override fun observeAll(): Flow<List<Note>> =
        noteDao.observeAll().map { list -> list.map { it.toDomain() } }

    override suspend fun getById(id: UUID): Note? =
        noteDao.getById(id.toString())?.toDomain()

    override suspend fun save(note: Note) =
        noteDao.upsert(note.toEntity())

    override suspend fun delete(id: UUID) =
        noteDao.deleteById(id.toString())

    override suspend fun getAll(): List<Note> =
        noteDao.getAll().map { it.toDomain() }
}
