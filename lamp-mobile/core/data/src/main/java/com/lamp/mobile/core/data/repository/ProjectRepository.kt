package com.lamp.mobile.core.data.repository

import com.lamp.mobile.core.model.Project
import kotlinx.coroutines.flow.Flow
import java.util.UUID

interface ProjectRepository {
    fun observeAll(): Flow<List<Project>>
    suspend fun getById(id: UUID): Project?
    suspend fun getByName(name: String): Project?
    suspend fun save(project: Project)
    suspend fun delete(id: UUID)
    suspend fun getAll(): List<Project>
}
