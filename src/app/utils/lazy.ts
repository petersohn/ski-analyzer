export function lazy<T>(func: () => T): () => T {
  const data: { value?: T } = {};
  return () => {
    if (data.value === undefined) {
      data.value = func();
    }
    return data.value;
  };
}
