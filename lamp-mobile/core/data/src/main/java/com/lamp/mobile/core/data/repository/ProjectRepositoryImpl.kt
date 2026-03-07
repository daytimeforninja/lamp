package com.lamp.mobile.core.data.repository

import com.lamp.mobile.core.database.converter.EntityMappers.toDomain
import com.lamp.mobile.core.database.converter.EntityMappers.toEntity
import com.lamp.mobile.core.database.dao.ProjectDao
import com.lamp.mobile.core.database.dao.TaskDao
import com.lamp.mobile.core.model.Project
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import java.util.UUID
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class ProjectRepositoryImpl @Inject constructor(
    private val projectDao: ProjectDao,
    private val taskDao: TaskDao,
) : ProjectRepository {

    override fun observeAll(): Flow<List<Project>> =
        combine(
            projectDao.observeAll(),
            taskDao.observeAll(),
        ) { projects, tasks ->
            val tasksByProject = tasks.groupBy { it.project }
            projects.map { entity ->
                val projectTasks = tasksByProject[entity.name]?.map { it.toDomain() } ?: emptyList()
                entity.toDomain(projectTasks)
            }
        }

    override suspend fun getById(id: UUID): Project? =
        projectDao.getById(id.toString())?.toDomain()

    override suspend fun getByName(name: String): Project? =
        projectDao.getByName(name)?.toDomain()

    override suspend fun save(project: Project) =
        projectDao.upsert(project.toEntity())

    override suspend fun delete(id: UUID) =
        projectDao.deleteById(id.toString())

    override suspend fun getAll(): List<Project> =
        projectDao.getAll().map { it.toDomain() }
}
