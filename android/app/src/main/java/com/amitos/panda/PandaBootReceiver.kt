package com.amitos.panda

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.util.Log

/**
 * Restarts Panda on device boot if it was previously enabled.
 */
class PandaBootReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action == Intent.ACTION_BOOT_COMPLETED) {
            Log.i("PandaAgent", "Boot completed — Panda service will auto-start via accessibility settings")
            // The accessibility service auto-restarts via Android's framework.
            // No explicit start needed; this receiver is for future foreground-service use.
        }
    }
}
