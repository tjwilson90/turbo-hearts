const EMPTY_ARRAY: never[] = [];

export function emptyArray<T>(): T[] {
  return EMPTY_ARRAY as T[];
}

export function removeAll<T>(arr: T[], toRemove: T[]) {
  const removeSet = new Set(toRemove);
  for (let i = 0; i < arr.length; i++) {
    if (removeSet.has(arr[i])) {
      arr.splice(i, 1);
      i--;
    }
  }
}

export function pushAll<T>(arr: T[], toAdd: T[]) {
  for (const add of toAdd) {
    arr.push(add);
  }
}
