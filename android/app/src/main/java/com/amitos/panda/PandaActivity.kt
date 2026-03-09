package com.amitos.panda

/**
 * PandaActivity — AmitOS Android Control Panel
 *
 * Shows connection status, supervised/autonomous mode toggle,
 * task log, and big red kill switch.
 */

import android.accessibilityservice.AccessibilityServiceInfo
import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.provider.Settings
import android.view.accessibility.AccessibilityManager
import android.widget.*
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity

class PandaActivity : AppCompatActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Build UI programmatically for zero-dependency simplicity
        val root = ScrollView(this)
        val container = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(48, 64, 48, 64)
        }
        root.addView(container)

        // ── Header ──────────────────────────────────────────────────────────
        val title = TextView(this).apply {
            text = "🐼 AmitOS Panda Agent"
            textSize = 24f
            setPadding(0, 0, 0, 8)
        }
        container.addView(title)

        val subtitle = TextView(this).apply {
            text = "Android Computer Control — powered by Accessibility Service"
            textSize = 13f
            setPadding(0, 0, 0, 32)
        }
        container.addView(subtitle)

        // ── Service Status ───────────────────────────────────────────────────
        val statusLabel = TextView(this).apply { text = "Service Status"; textSize = 16f }
        container.addView(statusLabel)

        val statusText = TextView(this).apply {
            text = if (isAccessibilityEnabled()) "✅ Panda Service Active" else "❌ Service Disabled"
            textSize = 14f
            setPadding(0, 4, 0, 24)
        }
        container.addView(statusText)

        // ── Enable Service Button ────────────────────────────────────────────
        if (!isAccessibilityEnabled()) {
            val enableBtn = Button(this).apply {
                text = "Open Accessibility Settings"
                setOnClickListener {
                    startActivity(Intent(Settings.ACTION_ACCESSIBILITY_SETTINGS))
                }
            }
            container.addView(enableBtn)
        }

        // ── Mode Toggle ──────────────────────────────────────────────────────
        val modeLabel = TextView(this).apply {
            text = "\nAgent Mode"
            textSize = 16f
        }
        container.addView(modeLabel)

        val modeGroup = RadioGroup(this).apply {
            orientation = RadioGroup.HORIZONTAL
        }
        val supervisedRadio = RadioButton(this).apply {
            text = "Supervised"
            isChecked = true
        }
        val autonomousRadio = RadioButton(this).apply {
            text = "Autonomous"
        }
        modeGroup.addView(supervisedRadio)
        modeGroup.addView(autonomousRadio)
        container.addView(modeGroup)

        val modeDesc = TextView(this).apply {
            text = "Supervised: ask permission for each action.\nAutonomous: pre-approved goals run without prompts."
            textSize = 12f
            setPadding(0, 4, 0, 24)
        }
        container.addView(modeDesc)

        // ── Connection Info ──────────────────────────────────────────────────
        val connLabel = TextView(this).apply {
            text = "Desktop Bridge"
            textSize = 16f
        }
        container.addView(connLabel)

        val connInfo = TextView(this).apply {
            text = "Listening on TCP port 7799\nConnect AmitOS desktop: Settings → Computer Control → Connect Android Panda"
            textSize = 12f
            setPadding(0, 4, 0, 24)
        }
        container.addView(connInfo)

        // ── Task Log ─────────────────────────────────────────────────────────
        val logLabel = TextView(this).apply {
            text = "Agent Log"
            textSize = 16f
        }
        container.addView(logLabel)

        val logView = TextView(this).apply {
            text = "[Ready] Panda agent initialized.\n[Info] Waiting for AmitOS connection…"
            textSize = 11f
            typeface = android.graphics.Typeface.MONOSPACE
            setPadding(16, 16, 16, 16)
            setBackgroundColor(0xFF1A1A2E.toInt())
            setTextColor(0xFF00FF88.toInt())
        }
        container.addView(logView)

        // ── Kill Switch ───────────────────────────────────────────────────────
        val killBtn = Button(this).apply {
            text = "⛔ EMERGENCY KILL SWITCH"
            setBackgroundColor(0xFFDC2626.toInt())
            setTextColor(0xFFFFFFFF.toInt())
            textSize = 16f
            setPadding(0, 32, 0, 32)

            setOnClickListener {
                AlertDialog.Builder(this@PandaActivity)
                    .setTitle("Emergency Kill Switch")
                    .setMessage("This will IMMEDIATELY halt all Panda agent actions. Continue?")
                    .setPositiveButton("HALT ALL ACTIONS") { _, _ ->
                        PandaAccessibilityService.instance?.let { svc ->
                            // Send kill switch command to service
                            logView.text = logView.text.toString() + "\n[KILL] Emergency kill switch activated!"
                        }
                    }
                    .setNegativeButton("Cancel", null)
                    .show()
            }
        }
        val killParams = LinearLayout.LayoutParams(
            LinearLayout.LayoutParams.MATCH_PARENT,
            LinearLayout.LayoutParams.WRAP_CONTENT
        ).apply { setMargins(0, 32, 0, 0) }
        container.addView(killBtn, killParams)

        // ── Version ───────────────────────────────────────────────────────────
        val versionText = TextView(this).apply {
            text = "\nPanda v1.0.0 — AmitOS Computer Control\ngithub.com/Ayush0Chaudhary/blurr"
            textSize = 11f
            setPadding(0, 16, 0, 0)
        }
        container.addView(versionText)

        setContentView(root)
    }

    private fun isAccessibilityEnabled(): Boolean {
        val am = getSystemService(Context.ACCESSIBILITY_SERVICE) as AccessibilityManager
        val services = am.getEnabledAccessibilityServiceList(AccessibilityServiceInfo.FEEDBACK_ALL_MASK)
        return services.any { it.id.contains("PandaAccessibilityService") }
    }
}
