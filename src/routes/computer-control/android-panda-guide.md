# Panda/blurr Android Agent — Build + Integration Guide

## Overview

The Panda agent is an Android Accessibility Service (based on Ayush0Chaudhary/blurr) that
allows AmitOS to control your Android phone exactly like a human — tapping, swiping, typing
in any app, reading screen content, and responding to voice commands from your PC.

## Architecture

```
AmitOS Desktop (Vy mode)
        │
        │  WebSocket + ADB bridge
        ▼
Android Phone (Panda/blurr APK)
  ├─ Accessibility Service (full phone control)
  ├─ Voice command relay (receives from desktop)
  ├─ Permission approval (sends to desktop)
  └─ Screen reader (sends state to desktop)
```

## Building the Panda APK

The APK integrates directly into the RalphHub Android shell. The key components:

### AndroidManifest.xml additions

```xml
<service
    android:name=".PandaAccessibilityService"
    android:exported="true"
    android:label="Panda Agent"
    android:permission="android.permission.BIND_ACCESSIBILITY_SERVICE">
    <intent-filter>
        <action android:name="android.accessibilityservice.AccessibilityService" />
    </intent-filter>
    <meta-data
        android:name="android.accessibilityservice"
        android:resource="@xml/panda_accessibility_service_config" />
</service>
```

### PandaAccessibilityService.kt (core)

```kotlin
class PandaAccessibilityService : AccessibilityService() {

    private val wsClient = PandaWebSocketClient("ws://YOUR_PC_IP:7788")

    override fun onAccessibilityEvent(event: AccessibilityEvent) {
        // Send screen state to PC
        wsClient.send(json {
            "type" to "screen_event"
            "eventType" to event.eventType
            "text" to event.text.joinToString()
            "packageName" to event.packageName
        })
    }

    override fun onServiceConnected() {
        wsClient.connect()
        wsClient.onCommand { cmd ->
            when (cmd.type) {
                "tap" -> performTap(cmd.x, cmd.y)
                "swipe" -> performSwipe(cmd.x1, cmd.y1, cmd.x2, cmd.y2)
                "input" -> performInput(cmd.text)
                "key" -> performKey(cmd.keyCode)
            }
        }
    }

    private fun performTap(x: Float, y: Float) {
        val path = Path().apply { moveTo(x, y) }
        val builder = GestureDescription.Builder()
            .addStroke(GestureDescription.StrokeDescription(path, 0, 100))
        dispatchGesture(builder.build(), null, null)
    }

    private fun performInput(text: String) {
        val args = Bundle().apply {
            putCharSequence(AccessibilityNodeInfo.ACTION_ARGUMENT_SET_TEXT_CHARSEQUENCE, text)
        }
        val focused = rootInActiveWindow?.findFocus(AccessibilityNodeInfo.FOCUS_INPUT)
        focused?.performAction(AccessibilityNodeInfo.ACTION_SET_TEXT, args)
    }
}
```

## Setup Steps

1. Build the APK: `cd android && ./gradlew assembleRelease`
2. Copy APK to RalphHub app data: `cp app/release/app-release.apk ~/.local/share/RalphHub/panda-agent.apk`
3. Connect Android device via USB with developer mode enabled
4. In RalphHub: Computer Control → Android tab → "Install Panda APK"
5. On phone: Settings → Accessibility → Panda Agent → Enable
6. Start an Android agent session from RalphHub

## Voice Command Flow

```
You (speaking to phone) → Google Assistant / Siri
    → Intent broadcast to Panda service
    → WebSocket relay to AmitOS desktop
    → Voice Assistant processes command
    → Response synced back to phone
```

## Security

- All ADB commands require USB connection (no remote ADB)
- Accessibility Service actions are logged to Memory Spine
- Permission mode "Ask" requires manual approval for each action
- Kill switch immediately disables all Panda sessions
