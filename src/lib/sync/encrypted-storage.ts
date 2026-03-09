/**
 * AES-GCM encrypted local storage for mobile-first offline data.
 * Uses Web Crypto API — available in Capacitor WebView (Android 6+).
 */

const ALGORITHM = 'AES-GCM';
const KEY_LENGTH = 256;
const ITERATIONS = 100_000;
const SALT_KEY = '__ralphhub_salt__';
const DB_VERSION = 1;

export class EncryptedStorage {
  private key: CryptoKey | null = null;
  private dbName: string;

  constructor(dbName = 'ralphhub_mobile') {
    this.dbName = dbName;
  }

  async init(passphrase: string): Promise<void> {
    const salt = await this.getSalt();
    this.key = await this.deriveKey(passphrase, salt);
  }

  private async getSalt(): Promise<Uint8Array> {
    const stored = localStorage.getItem(SALT_KEY);
    if (stored) {
      return new Uint8Array(JSON.parse(stored));
    }
    const salt = crypto.getRandomValues(new Uint8Array(16));
    localStorage.setItem(SALT_KEY, JSON.stringify(Array.from(salt)));
    return salt;
  }

  private async deriveKey(passphrase: string, salt: Uint8Array): Promise<CryptoKey> {
    const baseKey = await crypto.subtle.importKey(
      'raw',
      new TextEncoder().encode(passphrase),
      'PBKDF2',
      false,
      ['deriveKey']
    );
    return crypto.subtle.deriveKey(
      { name: 'PBKDF2', salt, iterations: ITERATIONS, hash: 'SHA-256' },
      baseKey,
      { name: ALGORITHM, length: KEY_LENGTH },
      false,
      ['encrypt', 'decrypt']
    );
  }

  async encrypt(data: unknown): Promise<string> {
    if (!this.key) throw new Error('EncryptedStorage not initialized');
    const iv = crypto.getRandomValues(new Uint8Array(12));
    const encoded = new TextEncoder().encode(JSON.stringify(data));
    const ciphertext = await crypto.subtle.encrypt({ name: ALGORITHM, iv }, this.key, encoded);
    const combined = new Uint8Array(iv.byteLength + ciphertext.byteLength);
    combined.set(iv, 0);
    combined.set(new Uint8Array(ciphertext), iv.byteLength);
    return btoa(String.fromCharCode(...combined));
  }

  async decrypt<T>(encoded: string): Promise<T> {
    if (!this.key) throw new Error('EncryptedStorage not initialized');
    const combined = Uint8Array.from(atob(encoded), (c) => c.charCodeAt(0));
    const iv = combined.slice(0, 12);
    const ciphertext = combined.slice(12);
    const plaintext = await crypto.subtle.decrypt({ name: ALGORITHM, iv }, this.key, ciphertext);
    return JSON.parse(new TextDecoder().decode(plaintext)) as T;
  }

  async set(key: string, value: unknown): Promise<void> {
    const encrypted = await this.encrypt(value);
    localStorage.setItem(`${this.dbName}_${key}`, encrypted);
  }

  async get<T>(key: string): Promise<T | null> {
    const raw = localStorage.getItem(`${this.dbName}_${key}`);
    if (!raw) return null;
    try {
      return await this.decrypt<T>(raw);
    } catch {
      return null;
    }
  }

  async delete(key: string): Promise<void> {
    localStorage.removeItem(`${this.dbName}_${key}`);
  }

  async keys(): Promise<string[]> {
    const prefix = `${this.dbName}_`;
    return Object.keys(localStorage)
      .filter((k) => k.startsWith(prefix))
      .map((k) => k.slice(prefix.length));
  }
}

export const storage = new EncryptedStorage();
