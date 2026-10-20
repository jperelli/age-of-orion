// Tiny IndexedDB key-value store. Holds the autosave, settings, imported assets and mods,
// all locally in the player's browser (or the desktop app's webview).

const DB = "age-of-orion";
export type StoreName = "kv" | "assets" | "mods";

let dbp: Promise<IDBDatabase> | null = null;

function db(): Promise<IDBDatabase> {
  dbp ??= new Promise((resolve, reject) => {
    const req = indexedDB.open(DB, 1);
    req.onupgradeneeded = () => {
      for (const s of ["kv", "assets", "mods"]) req.result.createObjectStore(s);
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
  return dbp;
}

function run<T>(store: StoreName, mode: IDBTransactionMode, f: (s: IDBObjectStore) => IDBRequest<T> | void): Promise<T> {
  return db().then(
    (d) =>
      new Promise<T>((resolve, reject) => {
        const tx = d.transaction(store, mode);
        const req = f(tx.objectStore(store));
        tx.oncomplete = () => resolve(req ? req.result : (undefined as T));
        tx.onerror = () => reject(tx.error);
      }),
  );
}

export const get = <T>(store: StoreName, key: string) => run<T | undefined>(store, "readonly", (s) => s.get(key));
export const put = (store: StoreName, key: string, value: unknown) => run<IDBValidKey>(store, "readwrite", (s) => s.put(value, key));
export const del = (store: StoreName, key: string) => run<undefined>(store, "readwrite", (s) => s.delete(key));
export const clear = (store: StoreName) => run<undefined>(store, "readwrite", (s) => s.clear());
export const keys = (store: StoreName) => run<IDBValidKey[]>(store, "readonly", (s) => s.getAllKeys());

/** Writes many entries in one transaction. */
export function putMany(store: StoreName, entries: [string, unknown][]): Promise<void> {
  return run<undefined>(store, "readwrite", (s) => {
    for (const [k, v] of entries) s.put(v, k);
  }).then(() => undefined);
}
