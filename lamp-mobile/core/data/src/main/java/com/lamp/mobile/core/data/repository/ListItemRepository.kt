package com.lamp.mobile.core.data.repository

import com.lamp.mobile.core.model.ListItem
import com.lamp.mobile.core.model.ListKind
import kotlinx.coroutines.flow.Flow
import java.util.UUID

interface ListItemRepository {
    fun observeByKind(kind: ListKind): Flow<List<ListItem>>
    suspend fun getById(id: UUID): ListItem?
    suspend fun save(item: ListItem)
    suspend fun delete(id: UUID)
}
