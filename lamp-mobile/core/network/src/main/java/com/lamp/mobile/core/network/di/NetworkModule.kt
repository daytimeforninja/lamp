package com.lamp.mobile.core.network.di

import dagger.Module
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent

/**
 * Network clients are not provided as singletons since they depend on
 * user-configured credentials. Instead, they are created on-demand by
 * the SyncEngine using credentials from CredentialStore.
 */
@Module
@InstallIn(SingletonComponent::class)
object NetworkModule
