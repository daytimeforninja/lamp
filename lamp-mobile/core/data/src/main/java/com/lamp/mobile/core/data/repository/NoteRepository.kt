package com.lamp.mobile.core.data.repository

import com.lamp.mobile.core.model.Note
import kotlinx.coroutines.flow.Flow
import java.util.UUID

interface NoteRepository {
    fun observeAll(): Flow<List<Note>>
    suspend fun getById(id: UUID): Note?
    suspend fun save(note: Note)
    suspend fun delete(id: UUID)
    suspend fun getAll(): List<Note>
}
