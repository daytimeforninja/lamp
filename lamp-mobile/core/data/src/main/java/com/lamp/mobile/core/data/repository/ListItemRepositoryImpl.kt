package com.lamp.mobile.core.data.repository

import com.lamp.mobile.core.database.converter.EntityMappers.toDomain
import com.lamp.mobile.core.database.converter.EntityMappers.toEntity
import com.lamp.mobile.core.database.dao.ListItemDao
import com.lamp.mobile.core.model.ListItem
import com.lamp.mobile.core.model.ListKind
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import java.util.UUID
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class ListItemRepositoryImpl @Inject constructor(
    private val listItemDao: ListItemDao,
) : ListItemRepository {

    override fun observeByKind(kind: ListKind): Flow<List<ListItem>> =
        listItemDao.observeByKind(kind.name).map { list -> list.map { it.toDomain() } }

    override suspend fun getById(id: UUID): ListItem? =
        listItemDao.getById(id.toString())?.toDomain()

    override suspend fun save(item: ListItem) =
        listItemDao.upsert(item.toEntity())

    override suspend fun delete(id: UUID) =
        listItemDao.deleteById(id.toString())
}
