package com.lamp.mobile.core.data.credential

import android.content.Context
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class CredentialStore @Inject constructor(
    @ApplicationContext context: Context,
) {
    private val masterKey = MasterKey.Builder(context)
        .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
        .build()

    private val prefs = EncryptedSharedPreferences.create(
        context,
        "lamp_credentials",
        masterKey,
        EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
        EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
    )

    fun getServerUrl(service: String): String? = prefs.getString("${service}_url", null)
    fun setServerUrl(service: String, url: String) = prefs.edit().putString("${service}_url", url).apply()

    fun getUsername(service: String): String? = prefs.getString("${service}_username", null)
    fun setUsername(service: String, username: String) = prefs.edit().putString("${service}_username", username).apply()

    fun getPassword(service: String): String? = prefs.getString("${service}_password", null)
    fun setPassword(service: String, password: String) = prefs.edit().putString("${service}_password", password).apply()

    fun clearCredentials(service: String) {
        prefs.edit()
            .remove("${service}_url")
            .remove("${service}_username")
            .remove("${service}_password")
            .apply()
    }

    fun hasCredentials(service: String): Boolean =
        getServerUrl(service) != null && getUsername(service) != null && getPassword(service) != null
}
