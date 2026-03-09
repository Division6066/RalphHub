import type { CapacitorConfig } from '@capacitor/cli';

const config: CapacitorConfig = {
  appId: 'com.ralphhub.mobile',
  appName: 'RalphHub Mobile',
  webDir: 'build',
  bundledWebRuntime: false,
  server: {
    androidScheme: 'https',
    // For development: point to your desktop machine's IP
    // url: 'http://192.168.1.x:5173',
    cleartext: false,
  },
  android: {
    buildOptions: {
      keystorePath: 'ralphhub.keystore',
      keystoreAlias: 'ralphhub',
    },
    minSdkVersion: 26,
    targetSdkVersion: 34,
    compileSdkVersion: 34,
  },
  plugins: {
    SplashScreen: {
      launchShowDuration: 1500,
      backgroundColor: '#060816',
      androidSplashResourceName: 'splash',
      androidScaleType: 'CENTER_CROP',
    },
    StatusBar: {
      style: 'Dark',
      backgroundColor: '#060816',
    },
    Keyboard: {
      resize: 'body',
      style: 'dark',
      resizeOnFullScreen: true,
    },
  },
};

export default config;
