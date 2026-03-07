package com.lamp.mobile.sync

import android.app.Service
import android.content.Intent
import android.os.IBinder
import com.lamp.mobile.core.contacts.ContactsSyncAdapter
import com.lamp.mobile.core.data.repository.ContactRepository
import dagger.hilt.android.AndroidEntryPoint
import javax.inject.Inject

@AndroidEntryPoint
class ContactsSyncService : Service() {
    @Inject lateinit var contactRepository: ContactRepository

    private var syncAdapter: ContactsSyncAdapter? = null

    override fun onCreate() {
        super.onCreate()
        syncAdapter = ContactsSyncAdapter(applicationContext, true, contactRepository)
    }

    override fun onBind(intent: Intent?): IBinder = syncAdapter!!.syncAdapterBinder
}
